//! Utilities for generating the bindings for types that should be marshaled as a handle.

use proc_macro2::TokenStream;
use quote::*;
use syn::*;

pub fn quote_type_as_handle(ident: &Ident) -> syn::Result<TokenStream> {
    let name = ident.to_string();
    let describe_ident = format_describe_ident!(ident);
    let drop_ident = format_drop_ident!(ident);

    Ok(quote! {
        // Implement `Describe` for the exported type.
        impl cs_bindgen::shared::schematic::Describe for #ident {
            fn describe<E>(describer: E) -> Result<E::Ok, E::Error>
            where
                E: cs_bindgen::shared::schematic::Describer,
            {
                let describer = describer.describe_struct(cs_bindgen::shared::schematic::type_name!(#ident))?;
                cs_bindgen::shared::schematic::DescribeStruct::end(describer)
            }
        }

        // Implement `Abi` for the type and references to the type.

        impl cs_bindgen::abi::Abi for #ident {
            type Abi = *mut Self;

            fn into_abi(self) -> Self::Abi {
                std::boxed::Box::into_raw(std::boxed::Box::new(self))
            }

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                *std::boxed::Box::from_raw(abi)
            }
        }

        impl<'a> cs_bindgen::abi::Abi for &'a #ident {
            type Abi = Self;

            fn into_abi(self) -> Self::Abi {
                self
            }

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                abi
            }
        }

        impl<'a> cs_bindgen::abi::Abi for &'a mut #ident {
            type Abi = *mut #ident;

            fn into_abi(self) -> Self::Abi {
                self as *mut _
            }

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                &mut *abi
            }
        }

        // Export a function that describes the exported type.
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> std::boxed::Box<cs_bindgen::abi::RawString> {
            let export = cs_bindgen::shared::NamedType {
                name: #name.into(),
                binding_style: cs_bindgen::shared::BindingStyle::Handle,
                schema: cs_bindgen::shared::schematic::describe::<#ident>().expect("Failed to describe struct type"),
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }

        // Export a function that can be used for dropping an instance of the type.
        #[no_mangle]
        pub unsafe extern "C" fn #drop_ident(_: <#ident as cs_bindgen::abi::Abi>::Abi) {}
    })
}
