//! Utilities for generating the raw bindings to exported Rust functions.
//!
//! In C#, the raw binding to an exported Rust function is a `static extern`
//! function, using the `[DllImport]` attribute to load the corresponding function
//! from the Rust dylib. This module provides

use crate::generate::{class, quote_primitive_type, TypeMap};
use cs_bindgen_shared::{schematic::Schema, BindingStyle, Export};
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma};

pub fn quote_raw_binding(export: &Export, dll_name: &str, types: &TypeMap) -> TokenStream {
    match export {
        Export::Fn(export) => {
            let dll_import_attrib = quote_dll_import(dll_name, &export.binding);
            let binding_ident = format_ident!("{}", &*export.binding);
            let return_ty = quote_binding_return_type(&export.output);
            let args = quote_binding_args(export.inputs(), types);

            quote! {
                #dll_import_attrib
                internal static extern #return_ty #binding_ident(#args);
            }
        }

        Export::Method(export) => {
            let dll_import_attrib = quote_dll_import(dll_name, &export.binding);
            let binding_ident = format_ident!("{}", &*export.binding);
            let return_ty = quote_binding_return_type(&export.output);

            // TODO: Unify input handling for raw bindings. It shouldn't be necessary to
            // manually insert the receiver. The current blocker is that schematic can't
            // represent reference types, so we can't generate a full list of inputs that
            // includes the receiver.
            let mut args = quote_binding_args(export.inputs(), types);
            if export.receiver.is_some() {
                args.insert(0, quote! { void* self });
            }

            quote! {
                #dll_import_attrib
                internal static extern #return_ty #binding_ident(#args);
            }
        }

        Export::Named(export) => {
            if export.binding_style == BindingStyle::Handle {
                class::quote_drop_fn(&export.name, dll_name)
            } else {
                quote! {}
            }
        }
    }
}

pub fn quote_raw_arg(schema: &Schema, types: &TypeMap) -> TokenStream {
    match schema {
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

        Schema::String => quote! { RawCsString },

        Schema::Enum(schema) => {
            let export = types
                .get(&schema.name)
                .expect("Couldn't find exported type for enum");

            match export.binding_style {
                BindingStyle::Handle => quote! { void* },

                // For enums that are passed by value, the raw representation depends on whether or
                // not the enum carries additional data:
                //
                // * Data-carrying enums are represented as a `RawEnum<T>`, where `T` is a generated
                //   union type containing the data for the variant.
                // * C-like enums are represented as a single integer value. The type used for the
                //   discriminant is either specified directly in the schema, or defaults to
                //   `isize`/`IntPtr`.
                BindingStyle::Value => {
                    if schema.has_data() {
                        format_ident!("RawEnum<{}__RawArg>", &*export.name).to_token_stream()
                    } else {
                        schema
                            .repr
                            .map(quote_primitive_type)
                            .unwrap_or(quote! { IntPtr })
                    }
                }
            }
        }

        Schema::Struct(schema) => {
            let export = types
                .get(&schema.name)
                .expect("Couldn't find exported type for struct");

            match export.binding_style {
                BindingStyle::Handle => quote! { void* },

                BindingStyle::Value => todo!("Support passing structs by value"),
            }
        }

        // TODO: Add support for passing user-defined types to Rust.
        Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_)
        | Schema::Option(_)
        | Schema::Seq(_)
        | Schema::Tuple(_)
        | Schema::Map { .. } => todo!("Generate argument binding"),

        Schema::Unit | Schema::I128 | Schema::U128 => {
            unreachable!("Invalid types should have already been handled")
        }
    }
}

pub fn quote_binding_return_type(schema: &Schema) -> TokenStream {
    match schema {
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

        Schema::Unit => quote! { void },

        // TODO: Actually look up the referenced type in the set of exported types and
        // determine what style of binding is used for it (or if it even has valid bindings
        // at all). For now, the only supported binding style for user-defined types is to
        // treat them as a handle, so we hard code that case here.
        Schema::Struct(_) => quote! { void* },

        // TODO: Actually look up the referenced type to determine what style of binding is
        // being used and what the repr of the discriminant is. For now we only have support
        // for simple (C-like) enums without an explicit repr, so the raw value will always
        // be an `isize`.
        Schema::Enum(_) => quote! { IntPtr },

        // TODO: Add support for passing user-defined types out from Rust.
        Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_)
        | Schema::Option(_)
        | Schema::Seq(_)
        | Schema::Tuple(_)
        | Schema::Map { .. } => todo!("Generate return type binding"),

        Schema::I128 | Schema::U128 => {
            unreachable!("Invalid types should have already been handled")
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
    types: &TypeMap<'_>,
) -> Punctuated<TokenStream, Comma> {
    inputs
        .map(|(name, schema)| {
            let ident = format_ident!("{}", name);
            let ty = quote_raw_arg(schema, types);

            quote! { #ty #ident }
        })
        .collect()
}
