use self::{binding::*, func::*};
use crate::Opt;
use cs_bindgen_shared::Export;
use heck::*;
use quote::*;
use std::ffi::OsStr;

// mod class;
mod binding;
mod func;

pub fn generate_bindings(exports: Vec<Export>, opt: &Opt) -> Result<String, failure::Error> {
    // TODO: Add a validation pass to detect any invalid types (e.g. 128 bit integers,
    // `()` as an argument). This would remove the need to have graceful error handling
    // around those cases.

    let dll_name = opt
        .input
        .file_stem()
        .and_then(OsStr::to_str)
        .expect("Unable to get name of wasm file");

    let class_name = format_ident!("{}", dll_name.to_camel_case());

    // Generate the raw bindings for all exported items.
    let raw_bindings: Vec<_> = exports
        .iter()
        .map(|item| quote_raw_binding(item, dll_name))
        .collect::<Result<_, _>>()?;

    let mut fn_bindings = Vec::new();
    // let mut method_bindings = Vec::new();
    for decl in &exports {
        match decl {
            Export::Fn(decl) => fn_bindings.push(quote_wrapper_fn(decl)),
            Export::Struct(decl) => todo!("Generate struct binding"),
            Export::Method(decl) => todo!("Generate method binding"),
        }
    }

    let generated = quote! {
        using System;
        using System.Runtime.InteropServices;
        using System.Text;

        internal unsafe static class __bindings
        {
            [DllImport(
                #dll_name,
                EntryPoint = "__cs_bindgen_drop_string",
                CallingConvention = CallingConvention.Cdecl)]
            internal static extern void __cs_bindgen_drop_string(RustOwnedString raw);

            #( #raw_bindings )*
        }

        public class #class_name
        {
            #( #fn_bindings )*
        }

        // #( #method_bindings )*

        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct RustOwnedString
        {
            public byte* Ptr;
            public UIntPtr Length;
            public UIntPtr Capacity;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct RawCsString
        {
            public char* Ptr;
            public UIntPtr Length;

            public RawCsString(char* ptr, UIntPtr len)
            {
                Ptr = ptr;
                Length = len;
            }

            public RawCsString(char* ptr, int len)
            {
                Ptr = ptr;
                Length = (UIntPtr)len;
            }
        }
    };

    Ok(generated.to_string())
}
