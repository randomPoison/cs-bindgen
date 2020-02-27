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
            type Abi = u64;

            fn into_abi(self) -> Self::Abi {
                self as u64
            }
        }

        impl cs_bindgen::abi::FromAbi for #ident {
            type Abi = u64;

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                match abi {
                    $( #from_abi_patterns )*

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
