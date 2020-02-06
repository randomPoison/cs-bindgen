use std::{mem, slice};

pub mod prelude {
    pub use cs_bindgen_macro::*;
}

/// Generates helper functions that must only be generated once.
///
/// This macro must be called exactly once by the final crate that is being used
/// with `cs-bindgen`.
///
/// Ideally these functions should be declared directly in the `cs-bindgen` crate,
/// there's currently no way to ensure that symbols defined in dependencies get
/// exported correctly on all platforms. In practice, it seems like on Linux the
/// symbols are not exported. See https://github.com/rust-lang/rfcs/issues/2771 for
/// more information.
#[macro_export]
macro_rules! generate_static_bindings {
    () => {
        /// Drops a `CString` that has been passed to the .NET runtime.
        #[no_mangle]
        pub unsafe extern "C" fn __cs_bindgen_drop_string(raw: cs_bindgen::RawString) {
            let _ = raw.into_string();
        }
    };
}

/// Raw representation of a [`String`] compatible with FFI.
///
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RawString {
    pub ptr: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

impl RawString {
    pub fn from_string(mut string: String) -> Self {
        let raw = Self {
            ptr: string.as_mut_ptr(),
            len: string.len(),
            capacity: string.capacity(),
        };

        // Ensure that the string isn't de-allocated, effectively transferring ownership of
        // its data to the `RawString`.
        mem::forget(string);

        raw
    }

    /// Reconstructs the original string from its raw parts.
    ///
    /// # Safety
    ///
    /// `into_string` must only be called once per string instance. Calling it more than
    /// once on the same string will result in undefined behavior.
    pub unsafe fn into_string(self) -> String {
        String::from_raw_parts(self.ptr, self.len as usize, self.capacity as usize)
    }
}

impl From<String> for RawString {
    fn from(from: String) -> Self {
        Self::from_string(from)
    }
}

/// Raw representation of a `string` passed from C#.
///
/// C# strings are encoded as utf-16, so they're effectively passed to rust as a
/// `u16` slice. This struct contains the raw pieces necessary to reconstruct the
/// slice, and provides a helper method `into_string` to copy the data into a
/// `String`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RawCsString {
    pub ptr: *const u16,
    pub len: i32,
}

impl RawCsString {
    pub unsafe fn into_string(self) -> String {
        let chars = slice::from_raw_parts(self.ptr, self.len as usize);

        // TODO: Is a lossy conversion the thing to do here?
        String::from_utf16_lossy(chars)
    }
}
