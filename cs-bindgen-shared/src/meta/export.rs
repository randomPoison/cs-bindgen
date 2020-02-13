use crate::meta::{Func, Method, Struct};
use derive_more::From;
use serde::*;

/// An item exported from the Rust as a language binding.
#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum Export {
    Fn(Func),
    Struct(Struct),
    Method(Method),
}

impl Export {
    pub fn ident(&self) -> &str {
        match self {
            Export::Fn(item) => &item.ident,
            Export::Struct(item) => &item.ident,
            Export::Method(item) => item.ident(),
        }
    }
}
