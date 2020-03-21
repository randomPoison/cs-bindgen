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

use crate::abi::{RawSlice, RawString};

/// Drops a `CString` that has been passed to the .NET runtime.
pub unsafe fn __cs_bindgen_drop_string(raw: RawString) {
    let _ = raw.into_string();
}

/// Converts a C# string (i.e. a UTF-16 slice) into a Rust string.
pub unsafe fn __cs_bindgen_string_from_utf16(raw: RawSlice<u16>) -> RawString {
    raw.into_string()
        .expect("Failed to convert C# string to Rust string")
        .into()
}
