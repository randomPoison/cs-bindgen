use crate::Primitive;
use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnArg {
    ident: String,
    pub ty: Primitive,
}

impl FnArg {
    pub fn new(ident: String, ty: Primitive) -> Self {
        FnArg { ident, ty }
    }
}
