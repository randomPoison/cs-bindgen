use self::{class::*, func::*};
use crate::Opt;
use cs_bindgen_shared::*;
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use std::ffi::OsStr;
use syn::{punctuated::Punctuated, token::Comma, Ident};

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
        .map(|item| match item {
            BindgenItem::Fn(item) => quote_raw_binding(item, &item.generated_ident(), dll_name),

            BindgenItem::Method(item) => {
                quote_raw_binding(&item.method, &item.binding_ident(), dll_name)
            }

            BindgenItem::Struct(item) => {
                let binding_ident = item.drop_fn_ident();
                let entry_point = binding_ident.to_string();
                quote! {
                    [DllImport(
                        #dll_name,
                        EntryPoint = #entry_point,
                        CallingConvention = CallingConvention.Cdecl)]
                    internal static extern void #binding_ident(void* self);
                }
            }
        })
        .collect::<Vec<_>>();

    let mut fn_bindings = Vec::new();
    let mut method_bindings = Vec::new();
    for decl in &decls {
        match decl {
            BindgenItem::Fn(decl) => {
                fn_bindings.push(quote_wrapper_fn(decl, &decl.generated_ident()))
            }
            BindgenItem::Struct(decl) => method_bindings.push(quote_struct_binding(decl)),
            BindgenItem::Method(decl) => method_bindings.push(quote_method_binding(decl)),
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

pub fn quote_raw_binding(
    bindgen_fn: &BindgenFn,
    binding_ident: &Ident,
    dll_name: &str,
) -> TokenStream {
    let entry_point = binding_ident.to_string();

    let binding_return_ty = match bindgen_fn.ret {
        ReturnType::Default => quote! { void },
        ReturnType::SelfType => quote! { void* },
        ReturnType::Primitive(prim) => match prim {
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
        },
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
        binding_args.insert(0, quote! { void* self })
    }

    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern #binding_return_ty #binding_ident(#binding_args);
    }
}
