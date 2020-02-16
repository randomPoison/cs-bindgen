use crate::{
    encode::{Encode, Encoder},
    schema::*,
    TypeName,
};

pub struct SchemaEncoder;

impl<'a> Encoder for &'a mut SchemaEncoder {
    type Ok = Schema;
    type Error = ();

    fn encode_bool(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::Bool)
    }

    fn encode_i8(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::I8)
    }

    fn encode_i16(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::I16)
    }

    fn encode_i32(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::I32)
    }

    fn encode_i64(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::I64)
    }

    fn encode_i128(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::I128)
    }

    fn encode_u8(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::U8)
    }

    fn encode_u16(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::U16)
    }

    fn encode_u32(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::U32)
    }

    fn encode_u64(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::U64)
    }

    fn encode_u128(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::I128)
    }

    fn encode_f32(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::F32)
    }

    fn encode_f64(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::F64)
    }

    fn encode_char(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::Char)
    }

    fn encode_string(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::String)
    }

    fn encode_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::Unit)
    }

    fn encode_option<T>(self) -> Result<Self::Ok, Self::Error>
    where
        T: Encode,
    {
        let inner = T::encode(self)?;
        Ok(Schema::Option(Box::new(inner)))
    }

    fn encode_unit_struct(self, name: TypeName) -> Result<Self::Ok, Self::Error> {
        Ok(Schema::UnitStruct(UnitStruct { name }))
    }

    fn encode_newtype_struct<T>(self, name: TypeName) -> Result<Self::Ok, Self::Error>
    where
        T: Encode,
    {
        let inner = T::encode(self)?;
        Ok(Schema::NewtypeStruct(Box::new(NewtypeStruct {
            name: name.into(),
            inner,
        })))
    }

    fn encode_enum<T>(self, _name: TypeName) -> Result<Self::Ok, Self::Error>
    where
        T: Encode,
    {
        unimplemented!()
    }

    fn encode_tuple(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn encode_tuple_struct(self, _name: TypeName) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn encode_seq<T>(self) -> Result<Self::Ok, Self::Error>
    where
        T: Encode,
    {
        unimplemented!()
    }

    fn encode_map<K, V>(self) -> Result<Self::Ok, Self::Error>
    where
        K: Encode,
        V: Encode,
    {
        unimplemented!()
    }

    fn encode_struct(self, _name: TypeName) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
}
