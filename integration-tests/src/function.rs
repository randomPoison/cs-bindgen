use cs_bindgen::prelude::*;

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

#[cs_bindgen]
#[allow(bad_style)]
pub fn arg_name_test(
    simple: bool,
    long_param_name: i32,
    oddParamName: i32,
    name_with3OddCasing: i32,
    _leading_underscore: String,
) {
    let _ = (simple, long_param_name, oddParamName, name_with3OddCasing);
}
