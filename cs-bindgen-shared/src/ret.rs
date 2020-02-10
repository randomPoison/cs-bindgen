use crate::Primitive;
use serde::*;
use syn::{spanned::Spanned, Type};

/// The return type of a function marked with `#[cs_bindgen]`.
///
/// This enum is similar to the syn `ReturnType` enum, but provides an additional
/// `Primitive` variant. This allows us to specifically identify primitive types
/// that can be passed across the FFI boundary without additional marshalling (or at
/// least without the complexity of fully describing the type).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReturnType {
    Default,
    SelfType,
    Primitive(Primitive),
}

impl ReturnType {
    pub fn from_syn(ret: &syn::ReturnType) -> syn::Result<Self> {
        let inner = match ret {
            syn::ReturnType::Default => return Ok(ReturnType::Default),
            syn::ReturnType::Type(_, inner) => inner,
        };

        match &**inner {
            Type::Path(path) => Some(path),
            _ => None,
        }
        .and_then(|path| path.path.get_ident())
        .and_then(|ident| {
            if ident == "Self" {
                Some(ReturnType::SelfType)
            } else {
                Primitive::from_ident(ident).map(ReturnType::Primitive)
            }
        })
        .ok_or(syn::Error::new(
            inner.span(),
            "Unsupported return type, only primitive types and `String` are supported",
        ))
    }

    pub fn primitive(self) -> Option<Primitive> {
        match self {
            ReturnType::Primitive(prim) => Some(prim),
            _ => None,
        }
    }
}
