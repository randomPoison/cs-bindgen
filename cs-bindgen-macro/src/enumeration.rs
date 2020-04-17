use crate::{
    describe_named_type, quote_index_fn, quote_vec_drop_fn, reject_generics, value, BindingStyle,
};
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
    let has_fields = item
        .variants
        .iter()
        .any(|variant| !variant.fields.is_empty());

    let bindings = if has_fields {
        quote_complex_enum(&item)?
    } else {
        quote_simple_enum(&item)?
    };

    result.extend(bindings);

    // Export a function that describes the exported type.
    let ident = &item.ident;
    result.extend(describe_named_type(&ident, BindingStyle::Value));

    Ok(result)
}

fn quote_simple_enum(item: &ItemEnum) -> syn::Result<TokenStream> {
    let ident = &item.ident;

    // TODO: Check for a `#[repr(...)]` attribute and handle alternate types for the
    // discriminant.
    let discriminant_ty = quote! { isize };

    let const_ident = item
        .variants
        .iter()
        .map(|variant| {
            format_ident!(
                "__cs_bindgen_generated__{}__{}__DISCRIMINANT",
                ident,
                &variant.ident
            )
        })
        .collect::<Vec<_>>();

    // Generate constants for each discriminant of the enum. This is to handle arbitrary
    // expressions for discriminant values.
    let mut next_discriminant = quote! { 0 };
    let discriminant_expr = item.variants.iter().enumerate().map(|(index, variant)| {
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
        let const_ident = &const_ident[index];
        next_discriminant = quote! { #const_ident + 1 };

        expr
    });

    // Generate the match arms for the `from_abi` impl.
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

    let variant_name = item
        .variants
        .iter()
        .map(|variant| &variant.ident)
        .collect::<Vec<_>>();

    Ok(quote! {
        #(
            #[allow(bad_style)]
            const #const_ident: #discriminant_ty = #discriminant_expr;
        )*

        impl cs_bindgen::abi::Abi for #ident {
            type Abi = #discriminant_ty;

            fn as_abi(&self) -> Self::Abi {
                match self {
                    #(
                        Self::#variant_name => #const_ident,
                    )*
                }
            }

            fn into_abi(self) -> Self::Abi {
                match self {
                    #(
                        Self::#variant_name => #const_ident,
                    )*
                }
            }

            #[allow(bad_style)]
            unsafe fn from_abi(abi: Self::Abi) -> Self {
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
    let abi_union_ty = format_binding_ident!(ident);

    // TODO: Check the repr of the enum to determine the actual discriminant type.
    let discriminant_ty = quote! { isize };

    // Generate binding struct for each variant of the enum.
    let raw_variant_types = item.variants.iter().filter_map(|variant| {
        let abi_ident = format_ident!("{}__{}", abi_union_ty, variant.ident);

        // NOTE: No binding struct is generated for unit variants or struct/tuple-like
        // variants that don't actually contain data, since only the discriminant is
        // needed to restore it.
        if variant.fields.is_empty() {
            return None;
        }

        Some(value::quote_abi_struct(&abi_ident, &variant.fields))
    });

    let abi_union_fields = item.variants.iter().filter_map(|variant| {
        // NOTE: No binding struct is generated for unit variants or empty variants, since only
        // the discriminant is needed to restore it.
        if variant.fields == Fields::Unit || variant.fields.is_empty() {
            return None;
        }

        let field_ident = &variant.ident;
        let field_ty = format_ident!("{}__{}", abi_union_ty, variant.ident);

        Some(quote! {
            #field_ident: #field_ty
        })
    });

    let as_abi_match_arms = item.variants.iter().enumerate().map(|(index, variant)| {
        let variant_ident = &variant.ident;
        let discriminant = Literal::usize_unsuffixed(index);
        let abi_ident = format_ident!("{}__{}", abi_union_ty, variant.ident);

        let field_idents = variant
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| value::raw_field_ident(index, field));

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

        let convert_fields = value::as_abi_fields(&variant.fields, |index, field| {
            value::raw_field_ident(index, field).into_token_stream()
        });

        quote! {
            Self::#variant_ident #destructure => cs_bindgen::abi::RawEnum::new(
                #discriminant,
                #abi_union_ty {
                    #variant_ident: #abi_ident {
                        #convert_fields
                    },
                },
            )
        }
    });

    let into_abi_match_arms = item.variants.iter().enumerate().map(|(index, variant)| {
        let variant_ident = &variant.ident;
        let discriminant = Literal::usize_unsuffixed(index);
        let abi_ident = format_ident!("{}__{}", abi_union_ty, variant.ident);

        let field_idents = variant
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| value::raw_field_ident(index, field));

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

        let convert_fields = value::into_abi_fields(&variant.fields, |index, field| {
            value::raw_field_ident(index, field).into_token_stream()
        });

        quote! {
            Self::#variant_ident #destructure => cs_bindgen::abi::RawEnum::new(
                #discriminant,
                #abi_union_ty {
                    #variant_ident: #abi_ident {
                        #convert_fields
                    },
                },
            )
        }
    });

    let from_abi_match_arms = item.variants.iter().enumerate().map(|(index, variant)| {
        let variant_ident = &variant.ident;
        let discriminant = Literal::usize_unsuffixed(index);

        // Generate the logic for converting each field in the variant, then wrap it in the
        // appropriate type of braces based on the variant style.
        let populate_variant =
            value::from_abi_fields(&variant.fields, &quote! { abi.#variant_ident });
        let braces = match &variant.fields {
            Fields::Named { .. } => quote! { { #populate_variant } },
            Fields::Unnamed { .. } => quote! { ( # populate_variant ) },
            Fields::Unit => quote! {},
        };

        quote! {
            #discriminant => {
                let abi = abi.value.assume_init();
                Self::#variant_ident #braces
            }
        }
    });

    let index_fn = quote_index_fn(ident);
    let vec_drop_fn = quote_vec_drop_fn(ident);

    Ok(quote! {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #[allow(bad_style)]
        pub union #abi_union_ty {
            #( #abi_union_fields, )*
        }

        unsafe impl cs_bindgen::abi::AbiPrimitive for #abi_union_ty {}

        #( #raw_variant_types )*

        // Generate the `Abi` impl for the enum.
        impl cs_bindgen::abi::Abi for #ident {
            type Abi = cs_bindgen::abi::RawEnum<#discriminant_ty, #abi_union_ty>;

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                match abi.discriminant {
                    #( #from_abi_match_arms, )*

                    _ => panic!("Unknown discriminant {} for {}", abi.discriminant, stringify!(#ident)),
                }
            }

            fn as_abi(&self) -> Self::Abi {
                match self {
                    #( #as_abi_match_arms, )*
                }
            }

            fn into_abi(self) -> Self::Abi {
                match self {
                    #( #into_abi_match_arms, )*
                }
            }
        }

        #index_fn
        #vec_drop_fn
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
