use crate::func::*;
use proc_macro2::{Literal, TokenStream};
use quote::*;
use syn::*;

pub fn quote_enum_item(item: ItemEnum) -> syn::Result<TokenStream> {
    reject_generics(
        &item.generics,
        "Generic enums not supported with `#[cs_bindgen]`",
    )?;

    // Derive `Describe` for the enum.
    //
    // TODO: Move this into a dedicated derive macro for schematic.
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

    // Export a function that describes the exported type.
    let ident = &item.ident;
    let describe_ident = format_describe_ident!(ident);
    let name = ident.to_string();
    result.extend(quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> std::boxed::Box<cs_bindgen::abi::RawString> {
            let export = cs_bindgen::shared::Enum {
                name: #name.into(),
                schema: cs_bindgen::shared::schematic::describe::<#ident>().expect("Failed to describe enum type"),

                // NOTE: Currently we always pass enums by value. At some point we'll likely also
                // want to support exporting enums as handles, at which point we'll need to update
                // this bit of code generation.
                binding_style: cs_bindgen::shared::BindingStyle::Value,
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }
    });

    Ok(result)
}

fn quote_simple_enum(item: &ItemEnum) -> syn::Result<TokenStream> {
    let ident = &item.ident;

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
    })
}

fn quote_complex_enum(item: &ItemEnum) -> syn::Result<TokenStream> {
    let ident = &item.ident;
    let from_abi_union_ty = format_ident!("__cs_bindgen_generated_raw_FromAbi__{}", ident);
    let into_abi_union_ty = format_ident!("__cs_bindgen_generated_raw_IntoAbi__{}", ident);

    // TODO: Check the repr of the enum to determine the actual discriminant type.
    let discriminant_ty = quote! { isize };

    // Generate binding struct for each variant of the enum.
    let raw_variant_types = item.variants.iter().filter_map(|variant| {
        let from_abi_ident = format_ident!("{}__{}", from_abi_union_ty, variant.ident);
        let into_abi_ident = format_ident!("{}__{}", into_abi_union_ty, variant.ident);

        // NOTE: No binding struct is generated for unit variants or struct/tuple-like
        // variants that don't actually contain data, since only the discriminant is
        // needed to restore it.
        if variant.fields.is_empty() {
            return None;
        }

        // Extract the list of fields for the binding struct. The generated struct is the
        // same for both struct-like and tuple-like variants, though in the latter case we
        // have to manually generate names for the fields based on the index of the element.
        let (from_fields, into_fields) = variant.fields.iter().enumerate().fold(
            (Vec::new(), Vec::new()),
            |(mut from_fields, mut into_fields), (index, field)| {
                let field_ty = &field.ty;
                let field_ident = field
                    .ident
                    .as_ref()
                    .map(Clone::clone)
                    .unwrap_or_else(|| format_ident!("element_{}", index));

                from_fields.push(quote! {
                    #field_ident: <#field_ty as cs_bindgen::abi::FromAbi>::Abi
                });

                into_fields.push(quote! {
                    #field_ident: <#field_ty as cs_bindgen::abi::IntoAbi>::Abi
                });

                (from_fields, into_fields)
            },
        );

        Some(quote! {
            #[repr(C)]
            #[derive(Debug, Clone, Copy)]
            #[allow(bad_style)]
            pub struct #from_abi_ident {
                #( #from_fields, )*
            }

            #[repr(C)]
            #[derive(Debug, Clone, Copy)]
            #[allow(bad_style)]
            pub struct #into_abi_ident {
                #( #into_fields, )*
            }
        })
    });

    let from_abi_union_fields = item.variants.iter().filter_map(|variant| {
        // NOTE: No binding struct is generated for unit variants or empty variants, since only
        // the discriminant is needed to restore it.
        if variant.fields == Fields::Unit || variant.fields.is_empty() {
            return None;
        }

        let field_ident = &variant.ident;
        let field_ty = format_ident!("{}__{}", from_abi_union_ty, variant.ident);

        Some(quote! {
            #field_ident: #field_ty
        })
    });

    let into_abi_union_fields = item.variants.iter().filter_map(|variant| {
        // NOTE: No binding struct is generated for unit variants or empty variants, since only
        // the discriminant is needed to restore it.
        if variant.fields == Fields::Unit || variant.fields.is_empty() {
            return None;
        }

        let field_ident = &variant.ident;
        let field_ty = format_ident!("{}__{}", into_abi_union_ty, variant.ident);

        Some(quote! {
            #field_ident: #field_ty
        })
    });

    let into_abi_match_arms = item.variants.iter().enumerate().map(|(index, variant)| {
        let variant_ident = &variant.ident;
        let discriminant = Literal::usize_unsuffixed(index);
        let abi_ident = format_ident!("{}__{}", into_abi_union_ty, variant.ident);

        let field_idents = variant
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| field_ident(index, field));

        // Generate the destructuring expression for the fields of the variant.
        let destructure = match &variant.fields {
            Fields::Named { .. } => quote! { { #( #field_idents, )* } },
            Fields::Unnamed { .. } => quote! { ( #( #field_idents, )* ) },
            Fields::Unit => quote! {},
        };

        // For empty variants use `RawEnum::unit` to create an enum representation with just
        // a discriminant.
        if variant.fields.is_empty() {
            return quote! {
                Self::#variant_ident #destructure => cs_bindgen::abi::RawEnum::unit(#discriminant)
            };
        }

        let convert_fields = variant.fields.iter().enumerate().map(|(index, field)| {
            let field_ident = field_ident(index, field);

            quote! {
                #field_ident: cs_bindgen::abi::IntoAbi::into_abi(#field_ident)
            }
        });

        quote! {
            Self::#variant_ident #destructure => cs_bindgen::abi::RawEnum::new(
                #discriminant,
                #into_abi_union_ty {
                    #variant_ident: #abi_ident {
                        #( #convert_fields, )*
                    },
                },
            )
        }
    });

    let from_abi_match_arms = item.variants.iter().enumerate().map(|(index, variant)| {
        let variant_ident = &variant.ident;
        let discriminant = Literal::usize_unsuffixed(index);
        let abi_ident = format_ident!("{}__{}", from_abi_union_ty, variant.ident);

        let braces = match &variant.fields {
            Fields::Named { .. } => quote! { {} },
            Fields::Unnamed { .. } => quote! { () },
            Fields::Unit => quote! {},
        };

        if variant.fields.is_empty() {
            return quote! {
                #discriminant => Self::#variant_ident #braces
            };
        }

        let field_idents = variant
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| field_ident(index, field))
            .collect::<Vec<_>>();

        let populate_variant = match &variant.fields {
            Fields::Named { .. } => quote! {
                { #( #field_idents: cs_bindgen::abi::FromAbi::from_abi(#field_idents), )* }
            },

            Fields::Unnamed { .. } => quote! {
                ( #( cs_bindgen::abi::FromAbi::from_abi(#field_idents), )* )
            },

            Fields::Unit => unreachable!(),
        };

        quote! {
            #discriminant => {
                let #abi_ident { #( #field_idents, )* } = abi.value.assume_init().#variant_ident;
                Self::#variant_ident #populate_variant
            }
        }
    });

    Ok(quote! {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #[allow(bad_style)]
        pub union #from_abi_union_ty {
            #( #from_abi_union_fields, )*
        }

        #[repr(C)]
        #[derive(Clone, Copy)]
        #[allow(bad_style)]
        pub union #into_abi_union_ty {
            #( #into_abi_union_fields, )*
        }

        unsafe impl cs_bindgen::abi::AbiPrimitive for #from_abi_union_ty {}
        unsafe impl cs_bindgen::abi::AbiPrimitive for #into_abi_union_ty {}

        #( #raw_variant_types )*

        // Generate the `From/IntoAbi` impls for the enum.
        impl cs_bindgen::abi::FromAbi for #ident {
            type Abi = cs_bindgen::abi::RawEnum<#discriminant_ty, #from_abi_union_ty>;

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                match abi.discriminant {
                    #( #from_abi_match_arms, )*

                    _ => panic!("Unknown discriminant {} for {}", abi.discriminant, stringify!(#ident)),
                }
            }
        }

        impl cs_bindgen::abi::IntoAbi for #ident {
            type Abi = cs_bindgen::abi::RawEnum<#discriminant_ty, #into_abi_union_ty>;

            fn into_abi(self) -> Self::Abi {
                match self {
                    #( #into_abi_match_arms, )*
                }
            }
        }
    })
}

fn quote_describe_impl(item: &ItemEnum) -> syn::Result<TokenStream> {
    let ident = &item.ident;

    // Iterate over the enum variants and generate the describe logic for each one.
    let describe_variants = item.variants.iter().map(|variant| {
        let variant_name = variant.ident.to_string();

        match &variant.fields {
            // Unit variants are described with a single call to `describe_unit_variant`. We
            // also need to pass in the value of the discriminant, if one was specified.
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

            // For tuple variants, we generate initially call `start_tuple_variant` and then
            // generate a call to `describe_element` for each element in the tuple.
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

            // For struct variants, we generate initially call `start_struct_variant` and
            // then generate a call to `describe_field` for each field.
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

fn field_ident(index: usize, field: &Field) -> Ident {
    field
        .ident
        .as_ref()
        .map(Clone::clone)
        .unwrap_or_else(|| format_ident!("element_{}", index))
}
