use cs_bindgen::{abi::Abi, prelude::*};
use pretty_assertions::assert_eq;

#[cs_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructWithArrays {
    pub vec_field: Vec<u32>,
    pub array_field: [i32; 4],
}

#[cs_bindgen]
impl StructWithArrays {
    // TODO: Add a method that returns the vec field as a slice, once slices are supported.
    pub fn get_vec(&self) -> Vec<u32> {
        self.vec_field.clone()
    }

    pub fn get_array(&self) -> [i32; 4] {
        self.array_field
    }
}

#[test]
fn struct_round_trip() {
    let original = StructWithArrays {
        vec_field: vec![1, 2, 3, 4],
        array_field: [1, 2, 3, 4],
    };
    let result: StructWithArrays = unsafe { Abi::from_abi(original.clone().into_abi()) };
    assert_eq!(original, result);
}

#[test]
fn int_vec_round_trip() {
    let original: Vec<u32> = vec![1, 2, 3, 4, 5, 6];
    let result: Vec<u32> = unsafe { Abi::from_abi(original.clone().into_abi()) };
    assert_eq!(original, result);
}

#[test]
fn str_vec_round_trip() {
    let original: Vec<&'static str> = vec!["foo", "bar", "baz"];
    let result: Vec<&'static str> = unsafe { Abi::from_abi(original.clone().into_abi()) };
    assert_eq!(original, result);
}

#[test]
fn string_vec_round_trip() {
    let original: Vec<String> = vec!["foo".into(), "bar".into(), "baz".into()];
    let result: Vec<String> = unsafe { Abi::from_abi(original.clone().into_abi()) };
    assert_eq!(original, result);
}

#[test]
fn int_array_round_trip() {
    let original: [u32; 4] = [1, 2, 3, 4];
    let result: [u32; 4] = unsafe { Abi::from_abi(original.clone().into_abi()) };
    assert_eq!(original, result);
}

#[test]
fn str_array_round_trip() {
    let original: [&'static str; 3] = ["foo", "bar", "baz"];
    let result: [&'static str; 3] = unsafe { Abi::from_abi(original.clone().into_abi()) };
    assert_eq!(original, result);
}

#[test]
fn string_array_round_trip() {
    let original: [String; 3] = ["foo".into(), "bar".into(), "baz".into()];
    let result: [String; 3] = unsafe { Abi::from_abi(original.clone().into_abi()) };
    assert_eq!(original, result);
}
