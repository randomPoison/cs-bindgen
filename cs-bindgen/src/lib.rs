pub mod abi;

// Re-export the shared backend so that it can be accessed from generated code.
pub use cs_bindgen_shared as shared;

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
        pub unsafe extern "C" fn __cs_bindgen_drop_string(
            raw: cs_bindgen::shared::abi::RawVec<u8>,
        ) {
            let _ = raw.into_string();
        }
    };
}
