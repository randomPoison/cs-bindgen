use std::{ffi::CString, os::raw::c_char};

pub mod prelude {
    pub use cs_bindgen_macro::*;
}

/// Drops a `CString` that has been passed to the .NET runtime.
#[no_mangle]
pub unsafe extern "C" fn __cs_bindgen_drop_string(raw: *mut c_char) {
    let _ = CString::from_raw(raw);
}
