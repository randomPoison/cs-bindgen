use cs_bindgen::prelude::*;

pub mod collections;
pub mod copy_types;
pub mod data_enum;
pub mod name_collision;
pub mod simple_enum;
pub mod structs;

// Re-export core cs_bindgen functionality. Required in order for the generated Wasm module.
cs_bindgen::export!();

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

#[cs_bindgen]
pub fn is_seven(value: i32) -> bool {
    value == 7
}

#[cs_bindgen]
pub fn void_return(test: i32) {
    println!("{}", test);
}
