use crate::func::*;
use proc_macro2::TokenStream;
use quote::*;
use syn::*;

pub fn quote_enum_item(item: ItemEnum) -> syn::Result<TokenStream> {
    reject_generics(
        &item.generics,
        "Generic enums not supported with `#[cs_bindgen]`",
    )?;

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

    if has_fields {
        quote_complex_enum(item)
    } else {
        quote_simple_enum(item)
    }
}

fn quote_simple_enum(item: ItemEnum) -> syn::Result<TokenStream> {
    let ident = item.ident;

    // TODO: Check for a `#[repr(...)]` attribute and handle alternate types for the
    // discriminant.
    let discriminant_ty = quote! { isize };

    // Generate constants for each discriminant of the enum. This is to handle arbitrary expressions for discriminant values.
    let mut prev_discriminant = None;
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
        // This matches the default behavior for how discriminant values are determined:
        // https://doc.rust-lang.org/reference/items/enumerations.html#custom-discriminant-values-for-field-less-enumerations
        let expr = variant
            .discriminant
            .as_ref()
            .map(|(_, expr)| expr.to_token_stream())
            .or_else(|| {
                prev_discriminant.take().map(|prev_discriminant| {
                    quote! {
                        #prev_discriminant + 1
                    }
                })
            })
            .unwrap_or(quote! { 0 });

        prev_discriminant = Some(const_ident.clone());

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
        // Implement `Describe` for the enum.
        impl cs_bindgen::shared::schematic::Describe for #ident {
            fn describe<E>(describer: E) -> Result<E::Ok, E::Error>
            where
                E: cs_bindgen::shared::schematic::Describer,
            {
                todo!("Describe the enum")
            }
        }

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

        // TODO: Generate the descriptor function.
    })
}

fn quote_complex_enum(_item: ItemEnum) -> syn::Result<TokenStream> {
    todo!("Generate bindings for data-carrying enum");
}
