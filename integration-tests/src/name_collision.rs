//! This module tests cases where generated types can potentially have name collisions.

use cs_bindgen::prelude::*;

#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Test {
    pub value: i32,
}

#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum TestEnum {
    Test(Test),
}
