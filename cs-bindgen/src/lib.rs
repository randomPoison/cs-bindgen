pub mod abi;

// Re-export the shared backend so that it can be accessed from generated code.
pub use cs_bindgen_shared as shared;

pub mod prelude {
    pub use cs_bindgen_macro::*;
}

/// Shared functionality that needs to be exported in the built dylib.
///
/// All symbols in this module MUST be re-exported in the final crate that is being
/// used with cs_bindgen. Put the following in the root module of your crate:
///
/// ```
/// pub use cs_bindgen::exports::*;
/// ```
///
/// Ideally users of this crate shouldn't need to do anything to re-export these
/// symbols, cs_bindgen should be able to handle this automatically. In practice, it
/// seems like on Linux the symbols are not exported. See https://github.com/rust-lang/rfcs/issues/2771
/// for more information.
pub mod exports {
    use crate::abi::RawVec;

    /// Drops a `CString` that has been passed to the .NET runtime.
    #[no_mangle]
    pub unsafe extern "C" fn __cs_bindgen_drop_string(raw: RawVec<u8>) {
        let _ = raw.into_string();
    }
}
