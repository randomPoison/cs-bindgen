use cs_bindgen::prelude::*;

cs_bindgen::generate_static_bindings!();

#[cs_bindgen]
pub fn greet_a_number(num: i32) -> String {
    format!("Hello, #{}!", num)
}

#[cs_bindgen]
pub fn return_a_number() -> i32 {
    7
}

#[cs_bindgen]
pub fn string_arg(arg: String) -> String {
    format!("Hello, {}!", arg)
}
