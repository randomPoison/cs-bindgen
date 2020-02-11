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

    let mut fn_bindings = Vec::new();
    let mut struct_bindings = Vec::new();
    for decl in &decls {
        match decl {
            BindgenItem::Fn(decl) => fn_bindings.push(quote_bindgen_fn(decl, dll_name)),
            BindgenItem::Struct(decl) => struct_bindings.push(quote_struct_binding(decl)),
            BindgenItem::Impl(decl) => struct_bindings.push(quote_impl_binding(decl, dll_name)),
        }
    }

    let generated = quote! {
        using System;
        using System.Runtime.InteropServices;
        using System.Text;

        public class #class_name
        {
            [DllImport(
                #dll_name,
                EntryPoint = "__cs_bindgen_drop_string",
                CallingConvention = CallingConvention.Cdecl)]
            private static extern void DropString(RustOwnedString raw);

            #( #fn_bindings )*

            [StructLayout(LayoutKind.Sequential)]
            private unsafe struct RustOwnedString
            {
                public byte* Ptr;
                public UIntPtr Length;
                public UIntPtr Capacity;
            }

            [StructLayout(LayoutKind.Sequential)]
            private unsafe struct RawCsString
            {
                public char* Ptr;
                public int Length;
            }
        }

        #( #struct_bindings )*
    };

    generated.to_string()
}
