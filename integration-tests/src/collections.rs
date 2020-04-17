//! Tests verifying that collection types (e.g. arrays and maps) can be used with C#.

use crate::simple_enum::SimpleCEnum;
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
pub struct HandleStruct {
    pub bar: i32,
}

#[cs_bindgen]
pub fn return_handle_vec() -> Vec<HandleStruct> {
    vec![HandleStruct { bar: 33 }, HandleStruct { bar: 12345 }]
}

#[cs_bindgen]
impl HandleStruct {
    pub fn bar(&self) -> i32 {
        self.bar
    }
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
