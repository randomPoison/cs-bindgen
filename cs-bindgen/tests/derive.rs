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
