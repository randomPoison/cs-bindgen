use crate::{handle, has_derive_copy, reject_generics, value};
use proc_macro2::TokenStream;
use quote::*;
use syn::*;

/// Generates the bindings for an exported struct.
pub fn quote_struct_item(item: ItemStruct) -> syn::Result<TokenStream> {
    reject_generics(
        &item.generics,
        "Generic structs are not supported with `#[cs_bindgen]`",
    )?;

    let ident = item.ident;

    // Determine whether we should marshal the type as a handle or by value.
    if has_derive_copy(&item.attrs)? {
        handle::quote_type_as_handle(&ident)
    } else {
        let abi_struct = value::quote_abi_struct(&abi_struct_ident, &item.fields);
        let describe_fn = describe_named_type(&item.ident, BindingStyle::Value);

        Ok(quote! {
            #abi_struct
            #describe_fn
        })
    }
}
