use cs_bindgen::prelude::*;

#[cs_bindgen]
pub fn greet_a_number(num: u32) -> String {
    format!("Hello, #{}!", num)
}
