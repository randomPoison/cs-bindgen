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

#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct StructWithMethods {
    pub foo: String,
}

#[cs_bindgen]
impl StructWithMethods {
    pub fn new(foo: String) -> StructWithMethods {
        Self { foo }
    }

    pub fn foo(&self) -> String {
        self.foo.clone()
    }
}

#[cs_bindgen]
#[derive(Debug, Clone)]
pub struct AnotherStructWithMethods {
    pub foo: String,
}

#[cs_bindgen]
impl AnotherStructWithMethods {
    pub fn new(foo: String) -> AnotherStructWithMethods {
        Self { foo }
    }

    pub fn foo(&self) -> String {
        self.foo.clone()
    }
}
