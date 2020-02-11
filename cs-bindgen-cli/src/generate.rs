use self::{class::*, func::*};
use crate::Opt;
use cs_bindgen_shared::*;
use heck::*;
use quote::*;
use std::ffi::OsStr;

mod class;
mod func;

pub fn generate_bindings(decls: Vec<BindgenItem>, opt: &Opt) -> String {
    let dll_name = opt
        .input
        .file_stem()
        .and_then(OsStr::to_str)
        .expect("Unable to get name of wasm file");

    let class_name = format_ident!("{}", dll_name.to_camel_case());

    let raw_bindings = decls
        .iter()
        .filter_map(|item| match item {
            BindgenItem::Fn(item) => Some(quote_raw_binding(item, dll_name)),
            BindgenItem::Method(item) => Some(quote_raw_binding(&item.method, dll_name)),

            // No raw bindings needed for structs, the drop function is handled separately.
            BindgenItem::Struct(_) => None,
        })
        .collect::<Vec<_>>();

    let mut fn_bindings = Vec::new();
    let mut method_bindings = Vec::new();
    for decl in &decls {
        match decl {
            BindgenItem::Fn(decl) => fn_bindings.push(quote_wrapper_fn(decl)),
            BindgenItem::Struct(decl) => method_bindings.push(quote_struct_binding(decl)),
            BindgenItem::Method(decl) => method_bindings.push(quote_method_binding(decl)),
        }
    }

    let generated = quote! {
        using System;
        using System.Runtime.InteropServices;
        using System.Text;

        internal static class __bindings
        {
            [DllImport(
                #dll_name,
                EntryPoint = "__cs_bindgen_drop_string",
                CallingConvention = CallingConvention.Cdecl)]
            private static extern void DropString(RustOwnedString raw);

            #( #raw_bindings )*
        }

        public class #class_name
        {
            #( #fn_bindings )*
        }

        #( #method_bindings )*

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
            public int Length;
        }
    };

    generated.to_string()
}
