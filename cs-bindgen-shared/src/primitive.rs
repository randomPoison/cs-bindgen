use serde::*;
use syn::*;

/// A "known" Rust type that can be directly marshalled across the FFI boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Primitive {
    String,
    Char,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
}

impl Primitive {
    pub fn from_ident(ident: &Ident) -> Option<Self> {
        Some(match &*ident.to_string() {
            "String" => Primitive::String,
            "char" => Primitive::Char,
            "i8" => Primitive::I8,
            "i16" => Primitive::I16,
            "i32" => Primitive::I32,
            "i64" => Primitive::I64,
            "u8" => Primitive::U8,
            "u16" => Primitive::U16,
            "u32" => Primitive::U32,
            "u64" => Primitive::U64,
            "f32" => Primitive::F32,
            "f64" => Primitive::F64,
            "bool" => Primitive::Bool,

            _ => return None,
        })
    }

    pub fn from_type(ty: &Type) -> Option<Self> {
        let ident = match &*ty {
            Type::Path(path) => match path.path.get_ident() {
                Some(ident) => ident,
                None => return None,
            },

            _ => return None,
        };

        Self::from_ident(ident)
    }
}
