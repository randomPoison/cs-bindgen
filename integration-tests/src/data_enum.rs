use crate::simple_enum::SimpleCEnum;
use cs_bindgen::prelude::*;

#[cs_bindgen]
#[derive(Debug, Clone)]
pub enum DataEnum {
    Foo,
    Bar(String),
    Baz { name: String, value: i32 },
    Coolness(InnerEnum),
    NestedStruct(InnerStruct),
}

#[cs_bindgen]
#[derive(Debug, Clone)]
pub enum InnerEnum {
    Cool(SimpleCEnum),
    Cooler(SimpleCEnum),
    Coolest(SimpleCEnum),
}

#[cs_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct InnerStruct {
    pub value: i32,
}

#[cs_bindgen]
pub fn roundtrip_data_enum(val: DataEnum) -> DataEnum {
    val
}

#[cs_bindgen]
pub fn generate_data_enum() -> DataEnum {
    DataEnum::Baz {
        name: "Randal".into(),
        value: 11,
    }
}
