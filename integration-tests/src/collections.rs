//! Tests verifying that collection types (e.g. arrays and maps) can be used with C#.

use crate::{
    data_enum::{DataEnum, InnerEnum},
    simple_enum::SimpleCEnum,
};
use cs_bindgen::prelude::*;

#[cs_bindgen]
pub fn return_vec_i8() -> Vec<i8> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_u8() -> Vec<u8> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_i16() -> Vec<i16> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_u16() -> Vec<u16> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_i32() -> Vec<i32> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_u32() -> Vec<u32> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_i64() -> Vec<i64> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_u64() -> Vec<u64> {
    vec![1, 2, 3, 4]
}

#[cs_bindgen]
pub fn return_vec_f32() -> Vec<f32> {
    vec![1.0, 2.1, 3.123, 4.00000004]
}

#[cs_bindgen]
pub fn return_vec_f64() -> Vec<f64> {
    vec![1.0, 2.1, 3.123, 4.00000004]
}

#[cs_bindgen]
pub fn return_vec_bool() -> Vec<bool> {
    vec![true, false, true, true]
}

#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct CopyStruct {
    pub bar: i32,
}

#[cs_bindgen]
pub fn return_struct_vec() -> Vec<CopyStruct> {
    vec![CopyStruct { bar: 33 }, CopyStruct { bar: 12345 }]
}

#[cs_bindgen]
pub fn return_simple_enum_vec() -> Vec<SimpleCEnum> {
    vec![SimpleCEnum::Foo, SimpleCEnum::Bar, SimpleCEnum::Baz]
}

#[cs_bindgen]
pub fn return_data_enum_vec() -> Vec<DataEnum> {
    vec![
        DataEnum::Foo,
        DataEnum::Bar("Cool string".into()),
        DataEnum::Coolness(InnerEnum::Coolest(SimpleCEnum::Foo)),
    ]
}
