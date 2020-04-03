use crate::data_enum::DataEnum;
use cs_bindgen::prelude::*;

// Basic struct with named parameters. Includes both primitive type fields and
// another struct field.
#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct PersonInfo {
    name: String,
    age: i32,
    address: Address,
}

// Export methods associated with an exported struct. Includes a constructor,
// getters, setters, and methods that operate on the internal state of the object.
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

// Test a struct with a field that is a data-carrying enum.
#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct WrapperType {
    pub value: DataEnum,
}

// Test tuple-like structs, including newtype structs (tuple-like structs with a single element).
#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct NewtypeStruct(u32);

#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct TupleStruct(String, String);
