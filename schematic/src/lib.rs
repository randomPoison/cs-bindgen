use encode::Encode;

mod encode;
mod primitive;
mod schema;
mod schema_encoder;

pub use crate::{encode::*, schema::*, schema_encoder::*};

/// Encodes `T` into an in-memory representation of the type tree.
pub fn encode<T: Encode>() -> Result<Schema, ()> {
    let mut encode = SchemaEncoder;
    T::encode(&mut encode)
}
