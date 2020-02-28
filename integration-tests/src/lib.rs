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
pub fn is_seven(value: i32) -> bool {
    value == 7
}

#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct PersonInfo {
    name: String,
    age: i32,
    address: Address,
}

#[cs_bindgen]
impl PersonInfo {
    // TODO: Change the return type back to `Self` once that's supported.
    pub fn new(name: String, age: i32) -> PersonInfo {
        Self {
            name,
            age,
            address: Address {
                street_number: 123,
                street: "Cool Kids Lane".into(),
            },
        }
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

    pub fn address(&self) -> Address {
        self.address.clone()
    }

    pub fn is_minor(&self) -> bool {
        self.age < 21
    }
}

#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct Address {
    street_number: u32,
    street: String,
}

#[cs_bindgen]
impl Address {
    pub fn street_number(&self) -> u32 {
        self.street_number
    }

    // TODO: Change this to return `&str` once that's supported.
    pub fn street_name(&self) -> String {
        self.street.clone()
    }
}

#[cs_bindgen]
pub fn void_return(test: i32) {
    println!("{}", test);
}

const DISCRIMINANT: isize = 45;

// TODO: Write a test that checks each of the variants and confirms that the
// `FromAbi` and `IntoAbi` impls agree on the discriminant values.
#[cs_bindgen]
pub enum SimpleEnum {
    Foo,
    Bar,
    Baz = 5,
    Baa,
    Bab,
    Quux = 1 + 2 + 3 + 4,
    Cool,
    Wool,
    SomeDiscriminant = DISCRIMINANT,
    AnotherOne,
    YetAnotherOne,
}

// #[cs_bindgen]
pub enum DataEnum {
    Foo,
    Bar(String),
    Baz { name: String, value: i32 },
}
