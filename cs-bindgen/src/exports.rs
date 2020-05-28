//! Shared functionality that needs to be exported in the built dylib.
//!
//! In order for the generated dylib to work on all platforms, you MUST invoke the
//! [`export`] macro once at the root of your crate:
//!
//! ```
//! cs_bindgen::export!();
//! ```
//!
//! Ideally users of this crate shouldn't need to do anything to re-export these
//! symbols, cs_bindgen should be able to handle this automatically. In practice, it
//! seems like on Linux the symbols are not exported. See https://github.com/rust-lang/rfcs/issues/2771
//! for more information.
//!
//! [`export`]: ../macro.export.html

use crate::abi::{self, Abi, RawSlice, RawString, RawVec};

macro_rules! drop_vec {
    ( $( $prim:ty => [$drop_fn:ident, $convert_fn:ident], )* ) => {
        $(
            pub unsafe fn $drop_fn(raw: RawVec<$prim>) {
                let _ = raw.into_vec();
            }

            pub unsafe fn $convert_fn(raw: RawSlice<<$prim as Abi>::Abi>) -> RawVec<$prim> {
                abi::convert_list(raw)
            }
        )*
    }
}

drop_vec! {
    u8 => [__cs_bindgen_drop_vec_u8, __cs_bindgen_convert_vec_u8],
    u16 => [__cs_bindgen_drop_vec_u16, __cs_bindgen_convert_vec_u16],
    u32 => [__cs_bindgen_drop_vec_u32, __cs_bindgen_convert_vec_u32],
    u64 => [__cs_bindgen_drop_vec_u64, __cs_bindgen_convert_vec_u64],
    usize => [__cs_bindgen_drop_vec_usize, __cs_bindgen_convert_vec_usize],

    i8 => [__cs_bindgen_drop_vec_i8, __cs_bindgen_convert_vec_i8],
    i16 => [__cs_bindgen_drop_vec_i16, __cs_bindgen_convert_vec_i16],
    i32 => [__cs_bindgen_drop_vec_i32, __cs_bindgen_convert_vec_i32],
    i64 => [__cs_bindgen_drop_vec_i64, __cs_bindgen_convert_vec_i64],
    isize => [__cs_bindgen_drop_vec_isize, __cs_bindgen_convert_vec_isize],

    f32 => [__cs_bindgen_drop_vec_f32, __cs_bindgen_convert_vec_f32],
    f64 => [__cs_bindgen_drop_vec_f64, __cs_bindgen_convert_vec_f64],

    bool => [__cs_bindgen_drop_vec_bool, __cs_bindgen_convert_vec_bool],
    char => [__cs_bindgen_drop_vec_char, __cs_bindgen_convert_vec_char],
}

/// Converts a C# string (i.e. a UTF-16 slice) into a Rust string.
pub unsafe fn __cs_bindgen_string_from_utf16(raw: RawSlice<u16>) -> RawString {
    raw.into_string()
        .expect("Failed to convert C# string to Rust string")
        .into()
}
