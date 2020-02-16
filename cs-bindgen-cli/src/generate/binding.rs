//! Utilities for generating the raw bindings to exported Rust functions.
//!
//! In C#, the raw binding to an exported Rust function is a `static extern`
//! function, using the `[DllImport]` attribute to load the corresponding function
//! from the Rust dylib. This module provides

use cs_bindgen_shared::{schematic::Schema, Export};
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma};

pub fn quote_raw_binding(export: &Export, dll_name: &str) -> Result<TokenStream, failure::Error> {
    match export {
        Export::Fn(item) => {
            let dll_import_attrib = quote_dll_import(dll_name, &item.binding);
            let binding_ident = format_ident!("{}", &*item.name);
            let return_ty = quote_binding_return_type(&item.output)?;
            let args = quote_binding_args(item.inputs())?;

            Ok(quote! {
                #dll_import_attrib
                internal static extern #return_ty #binding_ident(#args);
            })
        }

        Export::Method(item) => {
            let dll_import_attrib = quote_dll_import(dll_name, &item.binding);
            let binding_ident = format_ident!("{}", &*item.name);
            let return_ty = quote_binding_return_type(&item.output)?;
            let args = quote_binding_args(item.inputs())?;

            Ok(quote! {
                #dll_import_attrib
                internal static extern #return_ty #binding_ident(void* self, #args);
            })
        }

        Export::Struct(item) => {
            todo!("Binding for handle drop fn")
            // let binding_ident = item.drop_fn_ident();
            // let entry_point = binding_ident.to_string();
            // quote! {
            //     [DllImport(
            //         #dll_name,
            //         EntryPoint = #entry_point,
            //         CallingConvention = CallingConvention.Cdecl)]
            //     internal static extern void #binding_ident(void* self);
            // }
        }
    }
}

fn quote_dll_import(dll_name: &str, entry_point: &str) -> TokenStream {
    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
    }
}

fn quote_binding_args<'a>(
    inputs: impl Iterator<Item = (&'a str, &'a Schema)>,
) -> Result<Punctuated<TokenStream, Comma>, failure::Error> {
    let result = inputs
        .map(|(name, schema)| {
            let ident = format_ident!("{}", name);
            let ty = match schema {
                Schema::I8 => quote! { sbyte },
                Schema::I16 => quote! { short },
                Schema::I32 => quote! { int },
                Schema::I64 => quote! { long },
                Schema::U8 => quote! { byte },
                Schema::U16 => quote! { ushort },
                Schema::U32 => quote! { uint },
                Schema::U64 => quote! { ulong },
                Schema::F32 => quote! { float },
                Schema::F64 => quote! { double },
                Schema::Bool => quote! { byte },
                Schema::Char => quote! { uint },

                // `String` is passed to Rust as a `RawCsString`.
                Schema::String => quote! { __bindings.RawCsString },

                Schema::I128 | Schema::U128 => {
                    return Err(failure::err_msg("128 bit integers are not supported by C#"))
                }

                Schema::Unit => {
                    return Err(failure::err_msg(
                        "`()` cannot be passed as an argument from C#",
                    ))
                }

                // TODO: Add support for passing user-defined types out from Rust.
                Schema::Struct(_) => todo!("Generate argument binding"),
                Schema::UnitStruct(_) => todo!("Generate argument binding"),
                Schema::NewtypeStruct(_) => todo!("Generate argument binding"),
                Schema::TupleStruct(_) => todo!("Generate argument binding"),
                Schema::Enum(_) => todo!("Generate argument binding"),
                Schema::Option(_) => todo!("Generate argument binding"),
                Schema::Seq(_) => todo!("Generate argument binding"),
                Schema::Tuple(_) => todo!("Generate argument binding"),
                Schema::Map { key, value } => todo!("Generate argument binding"),
            };

            Ok(quote! { #ty #ident })
        })
        .collect::<Result<_, _>>()?;

    Ok(result)
}

fn quote_binding_return_type(schema: &Schema) -> Result<TokenStream, failure::Error> {
    let ty = match schema {
        Schema::I8 => quote! { sbyte },
        Schema::I16 => quote! { short },
        Schema::I32 => quote! { int },
        Schema::I64 => quote! { long },
        Schema::U8 => quote! { byte },
        Schema::U16 => quote! { ushort },
        Schema::U32 => quote! { uint },
        Schema::U64 => quote! { ulong },
        Schema::F32 => quote! { float },
        Schema::F64 => quote! { double },
        Schema::Bool => quote! { byte },
        Schema::Char => quote! { uint },

        // `String` is returned from Rust as a `RustOwnedString`.
        Schema::String => quote! { __bindings.RustOwnedString },

        Schema::I128 | Schema::U128 => {
            return Err(failure::err_msg("128 bit integers are not supported by C#"))
        }

        Schema::Unit => quote! { void },

        // TODO: Add support for passing user-defined types out from Rust.
        Schema::Struct(_) => todo!("Generate return type binding"),
        Schema::UnitStruct(_) => todo!("Generate return type binding"),
        Schema::NewtypeStruct(_) => todo!("Generate return type binding"),
        Schema::TupleStruct(_) => todo!("Generate return type binding"),
        Schema::Enum(_) => todo!("Generate return type binding"),
        Schema::Option(_) => todo!("Generate return type binding"),
        Schema::Seq(_) => todo!("Generate return type binding"),
        Schema::Tuple(_) => todo!("Generate return type binding"),
        Schema::Map { key, value } => todo!("Generate return type binding"),
    };

    Ok(ty)
}
