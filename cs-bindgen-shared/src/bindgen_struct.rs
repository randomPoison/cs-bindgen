use proc_macro2::Span;
use serde::*;
use syn::{spanned::Spanned, Error, Ident, ItemStruct};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindgenStruct {
    ty_ident: String,
}

impl BindgenStruct {
    pub fn from_item(item: ItemStruct) -> syn::Result<Self> {
        // Return an error for generic structs.
        if !item.generics.params.is_empty() {
            return Err(Error::new(
                item.generics.span(),
                "Generic structs are not not supported with `#[cs_bindgen]`",
            ));
        }

        Ok(Self {
            ty_ident: item.ident.to_string(),
        })
    }

    pub fn raw_ident(&self) -> &str {
        &self.ty_ident
    }

    pub fn ident(&self) -> Ident {
        Ident::new(&self.ty_ident, Span::call_site())
    }
}
