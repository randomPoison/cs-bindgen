use cs_bindgen::prelude::*;

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
#[derive(Debug, Clone)]
pub struct PersonInfo {
    name: String,
    age: i32,
}

#[cs_bindgen]
impl PersonInfo {
    // TODO: Change the return type back to `Self` once that's supported.
    pub fn new(name: String, age: i32) -> PersonInfo {
        Self { name, age }
    }

    // TODO: Change this to return `&str` once that's supported.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn age(&self) -> i32 {
        self.age
    }

    pub fn set_age(&mut self, age: i32) {
        self.age = age;
    }

    pub fn static_function() -> i32 {
        7
    }
}

#[cs_bindgen]
pub fn void_return(test: i32) {
    println!("{}", test);
}
