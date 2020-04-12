//! Tests verifying that collection types (e.g. arrays and maps) can be used with C#.

use cs_bindgen::prelude::*;

// #[cs_bindgen]
pub fn return_vec() -> Vec<i32> {
    vec![1, 2, 3, 4]
}
