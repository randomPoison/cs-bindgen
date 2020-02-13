use crate::meta::{Func, Struct};
use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub strct: Struct,
    pub method: Func,
}

impl Method {
    pub fn new(strct: Struct, method: Func) -> Self {
        Self { strct, method }
    }

    pub fn ident(&self) -> &str {
        &self.method.ident
    }

    pub fn binding(&self) -> &str {
        &self.method.binding
    }
}
