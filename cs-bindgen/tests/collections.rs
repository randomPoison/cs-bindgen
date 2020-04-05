use cs_bindgen::prelude::*;

#[cs_bindgen]
#[derive(Debug)]
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
