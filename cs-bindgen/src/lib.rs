pub mod abi;
pub mod exports;

// Re-export crates used in the generated code.
pub use cs_bindgen_shared as shared;

pub mod prelude {
    pub use cs_bindgen_macro::*;
}

/// Exports additional bindings such that they are accessible in the built dylib.
///
/// This MUST be invoked once at the root of your crate:
///
/// ```
/// cs_bindgen::export!();
/// ```
///
/// See [the `exports` module](exports/index.html) for more information.
#[macro_export]
macro_rules! export {
    (fn $name:ident($( $arg:ident : $type:ty ),*) -> $ret:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name($( $arg : $type),*) -> $ret {
            $crate::exports::$name($( $arg ),*)
        }
    };

    (fn $name:ident($( $arg:ident : $type:ty ),*)) => {
        $crate::export!(fn $name($( $arg : $type),*) -> ());
    };

    () => {
        $crate::export!(fn __cs_bindgen_string_from_utf16(raw: $crate::abi::RawSlice<u16>) -> $crate::abi::RawString);

        $crate::export!(fn __cs_bindgen_drop_vec_u8(raw: $crate::abi::RawVec<u8>));
        $crate::export!(fn __cs_bindgen_drop_vec_u16(raw: $crate::abi::RawVec<u16>));
        $crate::export!(fn __cs_bindgen_drop_vec_u32(raw: $crate::abi::RawVec<u32>));
        $crate::export!(fn __cs_bindgen_drop_vec_u64(raw: $crate::abi::RawVec<u64>));
        $crate::export!(fn __cs_bindgen_drop_vec_usize(raw: $crate::abi::RawVec<usize>));
        $crate::export!(fn __cs_bindgen_drop_vec_i8(raw: $crate::abi::RawVec<i8>));
        $crate::export!(fn __cs_bindgen_drop_vec_i16(raw: $crate::abi::RawVec<i16>));
        $crate::export!(fn __cs_bindgen_drop_vec_i32(raw: $crate::abi::RawVec<i32>));
        $crate::export!(fn __cs_bindgen_drop_vec_i64(raw: $crate::abi::RawVec<i64>));
        $crate::export!(fn __cs_bindgen_drop_vec_isize(raw: $crate::abi::RawVec<isize>));
        $crate::export!(fn __cs_bindgen_drop_vec_f32(raw: $crate::abi::RawVec<f32>));
        $crate::export!(fn __cs_bindgen_drop_vec_f64(raw: $crate::abi::RawVec<f64>));
        $crate::export!(fn __cs_bindgen_drop_vec_bool(raw: $crate::abi::RawVec<bool>));
        $crate::export!(fn __cs_bindgen_drop_vec_char(raw: $crate::abi::RawVec<char>));

        $crate::export!(fn __cs_bindgen_convert_vec_u8(raw: $crate::abi::RawSlice<<u8 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<u8>);
        $crate::export!(fn __cs_bindgen_convert_vec_u16(raw: $crate::abi::RawSlice<<u16 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<u16>);
        $crate::export!(fn __cs_bindgen_convert_vec_u32(raw: $crate::abi::RawSlice<<u32 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<u32>);
        $crate::export!(fn __cs_bindgen_convert_vec_u64(raw: $crate::abi::RawSlice<<u64 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<u64>);
        $crate::export!(fn __cs_bindgen_convert_vec_usize(raw: $crate::abi::RawSlice<<usize as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<usize>);
        $crate::export!(fn __cs_bindgen_convert_vec_i8(raw: $crate::abi::RawSlice<<i8 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<i8>);
        $crate::export!(fn __cs_bindgen_convert_vec_i16(raw: $crate::abi::RawSlice<<i16 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<i16>);
        $crate::export!(fn __cs_bindgen_convert_vec_i32(raw: $crate::abi::RawSlice<<i32 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<i32>);
        $crate::export!(fn __cs_bindgen_convert_vec_i64(raw: $crate::abi::RawSlice<<i64 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<i64>);
        $crate::export!(fn __cs_bindgen_convert_vec_isize(raw: $crate::abi::RawSlice<<isize as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<isize>);
        $crate::export!(fn __cs_bindgen_convert_vec_f32(raw: $crate::abi::RawSlice<<f32 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<f32>);
        $crate::export!(fn __cs_bindgen_convert_vec_f64(raw: $crate::abi::RawSlice<<f64 as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<f64>);
        $crate::export!(fn __cs_bindgen_convert_vec_bool(raw: $crate::abi::RawSlice<<bool as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<bool>);
        $crate::export!(fn __cs_bindgen_convert_vec_char(raw: $crate::abi::RawSlice<<char as $crate::abi::Abi>::Abi>) -> $crate::abi::RawVec<char>);
    };
}
