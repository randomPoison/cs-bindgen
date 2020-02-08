use crate::Primitive;
use serde::*;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

/// The return type of a function marked with `#[cs_bindgen]`.
///
/// This enum is similar to the syn `ReturnType` enum, but provides an additional
/// `Primitive` variant. This allows us to specifically identify primitive types
/// that can be passed across the FFI boundary without additional marshalling (or at
/// least without the complexity of fully describing the type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReturnType {
    Default,
    Primitive(Primitive),
}

impl ReturnType {
    pub fn into_primitive(self) -> Option<Primitive> {
        match self {
            ReturnType::Default => None,
            ReturnType::Primitive(prim) => Some(prim),
        }
    }
}

impl Parse for ReturnType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ret: syn::ReturnType = input.parse()?;
        let inner = match ret {
            syn::ReturnType::Default => return Ok(ReturnType::Default),
            syn::ReturnType::Type(_, inner) => inner,
        };

        match Primitive::from_type(&inner) {
            Some(prim) => Ok(ReturnType::Primitive(prim)),
            None => Err(syn::Error::new(
                inner.span(),
                "Unsupported return type, only primitive types and `String` are supported",
            )),
        }
    }
}
