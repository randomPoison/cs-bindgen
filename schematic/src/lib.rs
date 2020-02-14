use encode::Encode;

mod primitive;
mod schema;
mod schema_encoder;

pub mod decode;
pub mod encode;

pub use crate::{schema::*, schema_encoder::*};

/// Encodes the schema for the specified type.
pub fn encode<T: Encode>() -> Result<Schema, ()> {
    let mut encode = SchemaEncoder;
    T::encode(&mut encode)
}
