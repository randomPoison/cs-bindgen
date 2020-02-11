use crate::{BindgenFn, BindgenStruct, ReturnType};
use proc_macro2::Span;
use serde::*;
use syn::Ident;

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

    pub fn binding_ident(&self) -> Ident {
        Ident::new(&self.binding_name, Span::call_site())
    }

    pub fn binding_ident_str(&self) -> &str {
        &self.binding_name
    }

    pub fn is_constructor(&self) -> bool {
        self.method.raw_ident() == "new"
            && self.method.receiver.is_none()
            && self.method.ret == ReturnType::SelfType
    }
}
