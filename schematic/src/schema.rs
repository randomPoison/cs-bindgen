use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Schema {
    Struct(Box<Struct>),
    UnitStruct(UnitStruct),
    NewtypeStruct(Box<NewtypeStruct>),
    TupleStruct(TupleStruct),
    Enum(Box<Enum>),
    Option(Box<Schema>),
    Seq(Box<Schema>),
    Tuple(Vec<Schema>),
    Map {
        key: Box<Schema>,
        value: Box<Schema>,
    },
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    String,
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitStruct {
    pub name: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NewtypeStruct {
    pub name: Cow<'static, str>,
    pub inner: Schema,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Struct;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Enum;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TupleStruct;
