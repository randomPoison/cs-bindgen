use cs_bindgen::prelude::*;

#[cs_bindgen]
pub struct NewtypeStruct(u32);

#[cs_bindgen]
#[derive(Clone, Copy)]
pub struct CopyNewtypeStruct(u32);

#[cs_bindgen]
pub struct TupleStruct(u32, String, bool);

#[cs_bindgen]
#[derive(Clone, Copy)]
pub struct CopyTupleStruct(u32, u8, bool);

#[cs_bindgen]
pub enum CLikeEnum {
    Foo,
    Bar,
    Baz,
}

#[cs_bindgen]
#[derive(Clone, Copy)]
pub enum CLikeEnumCopy {
    Foo,
    Bar,
    Baz,
}

#[cs_bindgen]
pub enum DataEnum {
    Foo,
    Bar(u32),
    Baz(u32, u8),
    Quux { one: u32, two: u8 },
}
