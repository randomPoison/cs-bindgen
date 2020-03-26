use cs_bindgen::prelude::*;

#[cs_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimpleCEnum {
    Foo,
    Bar,
    Baz,
}

#[cs_bindgen]
pub fn roundtrip_simple_enum(val: SimpleCEnum) -> SimpleCEnum {
    val
}

#[cs_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumWithDiscriminants {
    Hello,
    There = 5,
    How,
    Are,
    You = -12,
}

#[cs_bindgen]
pub fn roundtrip_simple_enum_with_discriminants(
    val: EnumWithDiscriminants,
) -> EnumWithDiscriminants {
    val
}
