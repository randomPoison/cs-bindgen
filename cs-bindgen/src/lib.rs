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
        $crate::export!(fn __cs_bindgen_drop_string(raw: $crate::abi::RawString));
        $crate::export!(fn __cs_bindgen_string_from_utf16(raw: $crate::abi::RawSlice<u16>) -> $crate::abi::RawString);
    };
}
