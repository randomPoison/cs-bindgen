use self::{binding::*, class::*, enumeration::*, func::*};
use crate::Opt;
use cs_bindgen_shared::{schematic::Primitive, Export};
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use std::ffi::OsStr;

mod binding;
mod class;
mod enumeration;
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
    let mut binding_items = Vec::new();
    for export in &exports {
        match export {
            Export::Fn(export) => fn_bindings.push(quote_wrapper_fn(
                &*export.name,
                &*export.binding,
                None,
                export.inputs(),
                &export.output,
            )),
            Export::Struct(export) => binding_items.push(quote_struct(export)),
            Export::Method(export) => binding_items.push(quote_method_binding(export)),
            Export::Enum(export) => binding_items.push(quote_enum_binding(export)),
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

        #( #binding_items )*

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

/// Quotes the C# type corresponding to the given Rust primitive.
///
/// # Panics
///
/// Panics for `I128` and `U128`, since C# does not natively support 128 bit
/// integers. In order to avoid panicking, all types used in generated bindings
/// should be validated at the beginning of code generation and an error should be
/// generated for any unsupported types.
fn quote_primitive_type(ty: Primitive) -> TokenStream {
    match ty {
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::Usize => quote! { UIntPtr },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::Isize => quote! { IntPtr },

        Primitive::I128 | Primitive::U128 => panic!("128 bit integers not supported"),
    }
}
