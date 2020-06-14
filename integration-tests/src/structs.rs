use crate::data_enum::DataEnum;
use cs_bindgen::prelude::*;

// Basic struct with named parameters. Includes both primitive type fields and
// another struct field.
#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct BasicStruct {
    pub foo: i32,
    pub bar: String,
    pub baz: bool,
}

// Test a struct with a field that is a data-carrying enum.
#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct WrapperType {
    pub value: DataEnum,
}

// Test tuple-like structs, including newtype structs (tuple-like structs with a single element).
#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct NewtypeStruct(u32);

#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct TupleStruct(String, String);

#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct CopyTupleStruct(i32, i32);

#[cs_bindgen]
pub fn round_trip_copy_tuple_struct(value: CopyTupleStruct) -> CopyTupleStruct {
    value
}

#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct CopyNewtypeStruct(i32);

#[cs_bindgen]
pub fn round_trip_copy_newtype_struct(value: CopyNewtypeStruct) -> CopyNewtypeStruct {
    value
}
