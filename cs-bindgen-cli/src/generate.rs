use crate::Opt;
use cs_bindgen_shared::*;
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use std::ffi::OsStr;
use syn::{punctuated::Punctuated, token::Comma, *};

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

fn quote_bindgen_fn(bindgen_fn: &BindgenFn, dll_name: &str) -> TokenStream {
    let entry_point = bindgen_fn.generated_name();
    let raw_binding = format_ident!("__{}", bindgen_fn.raw_ident().to_camel_case());
    let binding_return_ty = match bindgen_fn.ret.primitive() {
        None => quote! { void },
        Some(prim) => quote_primitive_binding_return(prim),
    };

    let mut binding_args = bindgen_fn
        .args
        .iter()
        .map(|arg| {
            let ident = arg.ident();
            let ty = quote_primitive_binding_arg(arg.ty);
            quote! { #ty #ident }
        })
        .collect::<Punctuated<_, Comma>>();

    if bindgen_fn.receiver.is_some() {
        binding_args.insert(0, quote! { *void self })
    }

    let wrapper_fn = quote_wrapper_fn(&bindgen_fn, &raw_binding);

    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
        private static extern #binding_return_ty #raw_binding(#binding_args);

        #wrapper_fn
    }
}

/// Quotes the C# type for an argument to the raw binding function.
fn quote_primitive_binding_arg(arg_ty: Primitive) -> TokenStream {
    match arg_ty {
        Primitive::String => quote! { RawCsString },
        Primitive::Char => quote! { uint },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::F32 => quote! { float },
        Primitive::F64 => quote! { double },
        Primitive::Bool => quote! { byte },
    }
}

/// Quotes the C# type for a binding function's return type.
fn quote_primitive_binding_return(return_ty: Primitive) -> TokenStream {
    match return_ty {
        Primitive::String => quote! { RustOwnedString },
        Primitive::Char => quote! { uint },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::F32 => quote! { float },
        Primitive::F64 => quote! { double },
        Primitive::Bool => quote! { byte },
    }
}

/// Quotes the idiomatic C# type corresponding to a given primitive type.
fn quote_primitive(return_ty: Primitive) -> TokenStream {
    match return_ty {
        Primitive::String => quote! { string },
        Primitive::Char => quote! { uint },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::F32 => quote! { float },
        Primitive::F64 => quote! { double },
        Primitive::Bool => quote! { bool },
    }
}

fn quote_wrapper_fn(bindgen_fn: &BindgenFn, raw_binding: &Ident) -> TokenStream {
    let cs_fn_name = format_ident!("{}", bindgen_fn.raw_ident().to_camel_case());
    let cs_return_ty = match bindgen_fn.ret.primitive() {
        None => quote! { void },
        Some(prim) => quote_primitive(prim),
    };

    // Build the list of arguments to the wrapper function.
    let mut args = bindgen_fn
        .args
        .iter()
        .map(|arg| {
            let ident = arg.ident();
            let ty = quote_primitive(arg.ty);
            quote! { #ty #ident }
        })
        .collect::<Punctuated<_, Comma>>();

    // Build the list of arguments to the wrapper function.
    let mut invoke_args = bindgen_fn
        .args
        .iter()
        .map(|arg| match arg.ty {
            // To pass a string to Rust, we convert it into a `RawCsString` with the fixed pointer.
            // The code for wrapping the body of the function in a `fixed` block is done below,
            // since we need to generate the contents of the block first.
            Primitive::String => {
                let arg_ident = arg.ident();
                let fixed_ident = format_ident!("__fixed_{}", arg.raw_ident()).into_token_stream();
                quote! {
                    new RawCsString() { Ptr = #fixed_ident, Length = #arg_ident.Length, }
                }
            }

            Primitive::Bool => {
                let ident = arg.ident();
                quote! { (#ident ? 1 : 0) }
            }

            _ => arg.ident().into_token_stream(),
        })
        .collect::<Punctuated<_, Comma>>();

    // Insert additional self parameter if the function has a receiver.
    if bindgen_fn.receiver.is_some() {
        args.insert(0, quote! { void* self });
        invoke_args.insert(0, quote! { _handle });
    }

    let invoke_expr = match bindgen_fn.ret.primitive() {
        None => quote! { #raw_binding(#invoke_args); },

        Some(prim) => {
            let invoke_expr = quote! { var rawResult = #raw_binding(#invoke_args); };

            let result_expr = match prim {
                Primitive::String => quote! {
                    string result = Encoding.UTF8.GetString(rawResult.Ptr, (int)rawResult.Length);
                    DropString(rawResult);
                    return result;
                },

                Primitive::Bool => quote! {
                    return rawResult != 0;
                },

                _ => quote! { return rawResult; },
            };

            quote! {
                #invoke_expr

                #result_expr
            }
        }
    };

    // Wrap the body of the function in `fixed` blocks for any parameters that need to
    // be passed as pointers to Rust (just strings for now). We use `Iterator::fold` to
    // generate a series of nested `fixed` blocks. This is very smart code and won't be
    // hard to maintain at all, I'm sure.
    let body = bindgen_fn
        .args
        .iter()
        .fold(invoke_expr, |body, arg| match arg.ty {
            Primitive::String => {
                let arg_ident = arg.ident();
                let fixed_ident = format_ident!("__fixed_{}", arg.raw_ident()).into_token_stream();
                quote! {
                    fixed (char* #fixed_ident = #arg_ident)
                    {
                        #body
                    }
                }
            }

            _ => body,
        });

    quote! {
        public static #cs_return_ty #cs_fn_name(#args)
        {
            unsafe {
                // TODO: Process args so they're ready to pass to the rust fn.

                #body
            }
        }
    }
}

fn quote_struct_binding(bindgen_struct: &BindgenStruct) -> TokenStream {
    let ident = bindgen_struct.ident();
    quote! {
        public unsafe partial class #ident
        {
            private void* _handle;
        }
    }
}

fn quote_impl_binding(bindgen_impl: &BindgenImpl, dll_name: &str) -> TokenStream {
    let ident = bindgen_impl.ty_ident();

    let methods = bindgen_impl
        .methods
        .iter()
        .map(|method| quote_bindgen_fn(method, dll_name));

    quote! {
        partial class #ident
        {
            #( #methods )*
        }
    }
}
