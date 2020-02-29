use crate::func::*;
use proc_macro2::TokenStream;
use quote::*;
use syn::*;

pub fn quote_enum_item(item: ItemEnum) -> syn::Result<TokenStream> {
    reject_generics(
        &item.generics,
        "Generic enums not supported with `#[cs_bindgen]`",
    )?;

    let mut result = quote_describe_impl(&item)?;

    // Check the variants to determine if we're dealing with a C-style enum or one that
    // carries additional data.
    let mut has_fields = false;
    for variant in &item.variants {
        match variant.fields {
            Fields::Unit => {}
            _ => {
                has_fields = true;
                break;
            }
        }
    }

    let bindings = if has_fields {
        quote_complex_enum(&item)?
    } else {
        quote_simple_enum(&item)?
    };

    result.extend(bindings);

    Ok(result)
}

fn quote_simple_enum(item: &ItemEnum) -> syn::Result<TokenStream> {
    let ident = &item.ident;
    let name = ident.to_string();
    let describe_ident = format_describe_ident!(ident);

    // TODO: Check for a `#[repr(...)]` attribute and handle alternate types for the
    // discriminant.
    let discriminant_ty = quote! { isize };

    // Generate constants for each discriminant of the enum. This is to handle arbitrary expressions for discriminant values.
    let mut next_discriminant = quote! { 0 };
    let discriminant_consts = item.variants.iter().map(|variant| {
        let const_ident = format_ident!(
            "__cs_bindgen_generated__{}__{}__DISCRIMINANT",
            ident,
            &variant.ident
        );

        // Generate the expression for the variant's discriminant:
        //
        // * If the variant has an explicit discriminant, reuse that expression in the constant.
        // * Otherwise, the discriminant is the previous discriminant plus 1.
        // * Otherwise the default is 0 (in the case where there was no previous discriminant).
        //
        // This matches the behavior for how discriminant values are determined:
        // https://doc.rust-lang.org/reference/items/enumerations.html#custom-discriminant-values-for-field-less-enumerations
        let expr = variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| expr.to_token_stream())
            .unwrap_or(next_discriminant.clone());

        // Generate the expression for the next variant's discriminant based on the
        // constant for the current variant.
        next_discriminant = quote! { #const_ident + 1 };

        quote! {
            const #const_ident: #discriminant_ty = #expr;
        }
    });

    // Generate the match arms for the `FromAbi` impl.
    let from_abi_patterns = item.variants.iter().map(|variant| {
        let const_ident = format_ident!(
            "__cs_bindgen_generated__{}__{}__DISCRIMINANT",
            ident,
            &variant.ident
        );
        let variant_ident = &variant.ident;

        quote! {
            #const_ident => #ident::#variant_ident
        }
    });

    Ok(quote! {
        // Implement `FromAbi` and `IntoAbi` for the enum.

        impl cs_bindgen::abi::IntoAbi for #ident {
            type Abi = #discriminant_ty;

            fn into_abi(self) -> Self::Abi {
                self as #discriminant_ty
            }
        }

        impl cs_bindgen::abi::FromAbi for #ident {
            type Abi = #discriminant_ty;

            #[allow(bad_style)]
            unsafe fn from_abi(abi: Self::Abi) -> Self {
                #( #discriminant_consts )*

                match abi {
                    #( #from_abi_patterns, )*

                    _ => panic!("Unknown variant {} for enum {}", abi, stringify!(#ident)),
                }
            }
        }

        // Export a function that describes the exported type.
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> std::boxed::Box<cs_bindgen::abi::RawString> {
            let export = cs_bindgen::shared::Enum {
                name: #name.into(),
                binding_style: cs_bindgen::shared::BindingStyle::Handle,
                schema: cs_bindgen::shared::schematic::describe::<#ident>().expect("Failed to describe enum type"),
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }
    })
}

fn quote_complex_enum(item: &ItemEnum) -> syn::Result<TokenStream> {
    let ident = &item.ident;
    // let name = ident.to_string();
    // let describe_ident = format_describe_ident!(ident);

    Ok(quote! {
        // TODO: Generate the `From/IntoAbi` impls for the enum.

        // TODO: Generate the descriptor function.
    })
}

fn quote_describe_impl(item: &ItemEnum) -> syn::Result<TokenStream> {
    let ident = &item.ident;

    let describe_variants = item.variants.iter().map(|variant| {
        let variant_name = variant.ident.to_string();

        match &variant.fields {
            Fields::Unit => {
                let discriminant = match &variant.discriminant {
                    Some((_, expr)) => quote! { Some((#expr).into()) },
                    None => quote! { None },
                };

                quote! {
                    cs_bindgen::shared::schematic::DescribeEnum::describe_unit_variant(
                        &mut describer,
                        #variant_name,
                        #discriminant,
                    )?;
                }
            },
            

            Fields::Unnamed(fields) => {
                let describe_elements = fields.unnamed.iter().map(|field| {
                    let ty = &field.ty;

                    quote! {
                        cs_bindgen::shared::schematic::DescribeTupleVariant::describe_element::<#ty>(
                            &mut variant_describer,
                        )?;
                    }
                });

                quote! {
                    {
                        let mut variant_describer = cs_bindgen::shared::schematic::DescribeEnum::start_tuple_variant(
                            &mut describer,
                            #variant_name,
                        )?;
                        #( #describe_elements )*
                        cs_bindgen::shared::schematic::DescribeEnum::end_tuple_variant(
                            &mut describer,
                            variant_describer,
                        )?;
                    }
                }
            }

            Fields::Named(fields) => {
                let describe_fields = fields.named.iter().map(|field| {
                    let name = field.ident.as_ref().unwrap().to_string();
                    let ty = &field.ty;

                    quote! {
                        cs_bindgen::shared::schematic::DescribeStructVariant::describe_field::<#ty>(
                            &mut variant_describer,
                            #name,
                        )?;
                    }
                });

                quote! {
                    {
                        let mut variant_describer = cs_bindgen::shared::schematic::DescribeEnum::start_struct_variant(
                            &mut describer,
                            #variant_name,
                        )?;
                        #( #describe_fields )*
                        cs_bindgen::shared::schematic::DescribeEnum::end_struct_variant(
                            &mut describer,
                            variant_describer,
                        )?;
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl cs_bindgen::shared::schematic::Describe for #ident {
            fn describe<E>(describer: E) -> Result<E::Ok, E::Error>
            where
                E: cs_bindgen::shared::schematic::Describer,
            {
                let mut describer = describer.describe_enum(
                    cs_bindgen::shared::schematic::type_name!(#ident),
                )?;
                #( #describe_variants )*
                cs_bindgen::shared::schematic::DescribeEnum::end(describer)
            }
        }
    })
}
