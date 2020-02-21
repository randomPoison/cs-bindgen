//! Utilities for generating the raw bindings to exported Rust functions.
//!
//! In C#, the raw binding to an exported Rust function is a `static extern`
//! function, using the `[DllImport]` attribute to load the corresponding function
//! from the Rust dylib. This module provides

use crate::generate::class;
use cs_bindgen_shared::{schematic::Schema, Export};
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma};

pub fn quote_raw_binding(export: &Export, dll_name: &str) -> Result<TokenStream, failure::Error> {
    match export {
        Export::Fn(item) => {
            let dll_import_attrib = quote_dll_import(dll_name, &item.binding);
            let binding_ident = format_ident!("{}", &*item.binding);
            let return_ty = quote_binding_return_type(&item.output)?;
            let args = quote_binding_args(item.inputs())?;

            Ok(quote! {
                #dll_import_attrib
                internal static extern #return_ty #binding_ident(#args);
            })
        }

        Export::Method(item) => {
            let dll_import_attrib = quote_dll_import(dll_name, &item.binding);
            let binding_ident = format_ident!("{}", &*item.binding);
            let return_ty = quote_binding_return_type(&item.output)?;

            // TODO: Unify input handling for raw bindings. It shouldn't be necessary to
            // manually insert the receiver. The current blocker is that schematic can't
            // represent refrence types, so we can't generate a full list of inputs that
            // includes the receiver.
            let mut args = quote_binding_args(item.inputs())?;
            if item.receiver.is_some() {
                args.insert(0, quote! { void* self });
            }

            Ok(quote! {
                #dll_import_attrib
                internal static extern #return_ty #binding_ident(#args);
            })
        }

        Export::Struct(item) => Ok(class::quote_drop_fn(&item.name, dll_name)),
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
                Schema::String => quote! { RawCsString },

                Schema::I128 | Schema::U128 => {
                    return Err(failure::err_msg("128 bit integers are not supported by C#"))
                }

                Schema::Unit => {
                    return Err(failure::err_msg(
                        "`()` cannot be passed as an argument from C#",
                    ))
                }

                // TODO: Add support for passing user-defined types out from Rust.
                Schema::Struct(_)
                | Schema::UnitStruct(_)
                | Schema::NewtypeStruct(_)
                | Schema::TupleStruct(_)
                | Schema::Enum(_)
                | Schema::Option(_)
                | Schema::Seq(_)
                | Schema::Tuple(_)
                | Schema::Map { .. } => todo!("Generate argument binding"),
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
        Schema::String => quote! { RustOwnedString },

        Schema::I128 | Schema::U128 => {
            return Err(failure::err_msg("128 bit integers are not supported by C#"))
        }

        Schema::Unit => quote! { void },

        // TODO: Actually look up the referenced type in the set of exported types and
        // determine what style of binding is used for it (or if it even has valid bindings
        // at all). For now, the only supported binding style for user-defined types is to
        // treat them as a handle, so we hard code that case here.
        Schema::Struct(_) => quote! { void* },

        // TODO: Add support for passing user-defined types out from Rust.
        Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_)
        | Schema::Enum(_)
        | Schema::Option(_)
        | Schema::Seq(_)
        | Schema::Tuple(_)
        | Schema::Map { .. } => todo!("Generate return type binding"),
    };

    Ok(ty)
}
