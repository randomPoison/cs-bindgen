use crate::Primitive;
use proc_macro2::Span;
use serde::*;
use syn::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnArg {
    ident: String,
    pub ty: Primitive,
}

impl FnArg {
    pub fn new(ident: String, ty: Primitive) -> Self {
        FnArg { ident, ty }
    }

    pub fn raw_ident(&self) -> &str {
        &self.ident
    }

    pub fn ident(&self) -> Ident {
        Ident::new(&self.ident, Span::call_site())
    }
}
