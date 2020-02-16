use crate::TypeName;

pub trait Encode: Sized {
    fn encode<E>(encoder: E) -> Result<E::Ok, E::Error>
    where
        E: Encoder;
}

pub trait Encoder: Sized {
    type Ok;
    type Error;

    fn encode_bool(self) -> Result<Self::Ok, Self::Error>;
    fn encode_i8(self) -> Result<Self::Ok, Self::Error>;
    fn encode_i16(self) -> Result<Self::Ok, Self::Error>;
    fn encode_i32(self) -> Result<Self::Ok, Self::Error>;
    fn encode_i64(self) -> Result<Self::Ok, Self::Error>;
    fn encode_i128(self) -> Result<Self::Ok, Self::Error>;
    fn encode_u8(self) -> Result<Self::Ok, Self::Error>;
    fn encode_u16(self) -> Result<Self::Ok, Self::Error>;
    fn encode_u32(self) -> Result<Self::Ok, Self::Error>;
    fn encode_u64(self) -> Result<Self::Ok, Self::Error>;
    fn encode_u128(self) -> Result<Self::Ok, Self::Error>;
    fn encode_f32(self) -> Result<Self::Ok, Self::Error>;
    fn encode_f64(self) -> Result<Self::Ok, Self::Error>;
    fn encode_char(self) -> Result<Self::Ok, Self::Error>;
    fn encode_string(self) -> Result<Self::Ok, Self::Error>;
    fn encode_unit(self) -> Result<Self::Ok, Self::Error>;

    fn encode_option<T>(self) -> Result<Self::Ok, Self::Error>
    where
        T: Encode;

    fn encode_unit_struct(self, name: TypeName) -> Result<Self::Ok, Self::Error>;

    fn encode_enum<T>(self, name: TypeName) -> Result<Self::Ok, Self::Error>
    where
        T: Encode;

    fn encode_newtype_struct<T>(self, name: TypeName) -> Result<Self::Ok, Self::Error>
    where
        T: Encode;

    fn encode_seq<T>(self) -> Result<Self::Ok, Self::Error>
    where
        T: Encode;

    fn encode_tuple(self) -> Result<Self::Ok, Self::Error>;

    fn encode_tuple_struct(self, name: TypeName) -> Result<Self::Ok, Self::Error>;

    fn encode_map<K, V>(self) -> Result<Self::Ok, Self::Error>
    where
        K: Encode,
        V: Encode;

    fn encode_struct(self, name: TypeName) -> Result<Self::Ok, Self::Error>;
}
