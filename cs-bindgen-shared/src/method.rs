use crate::{BindgenFn, BindgenStruct};
use serde::*;
use syn::Ident;
use proc_macro2::Span;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    binding_name: String,
    pub strct: BindgenStruct,
    pub method: BindgenFn,
}

impl Method {
    pub fn new(strct: BindgenStruct, method: BindgenFn) -> Self {
        Self {
            binding_name: format!(
                "__cs_bindgen_generated__{}__{}",
                strct.raw_ident(),
                method.raw_ident()
            ),
            strct,
            method,
        }
    }

    pub fn ident(&self) -> Ident {
        Ident::new(&self.binding_name, Span::call_site())
    }

    pub fn raw_ident(&self) -> &str {
        &self.binding_name
    }
}
