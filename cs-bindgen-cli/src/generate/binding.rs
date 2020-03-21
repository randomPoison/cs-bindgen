//! Utilities for generating the raw bindings to exported Rust functions.
//!
//! In C#, the raw binding to an exported Rust function is a `static extern`
//! function, using the `[DllImport]` attribute to load the corresponding function
//! from the Rust dylib. This module provides

use crate::generate::{class, enumeration, TypeMap};
use cs_bindgen_shared::{
    schematic::{Enum, Schema, Struct},
    BindingStyle, Export,
};
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma, Ident};

/// Generates the identifier for the from-raw conversion function for the specified type.
pub fn from_raw_ident(name: &str) -> Ident {
    format_ident!("__{}FromRaw", name)
}

/// Generates the identifier for the into-raw conversion function for the specified type.
pub fn into_raw_ident(name: &str) -> Ident {
    format_ident!("__{}IntoRaw", name)
}

pub fn quote_raw_binding(export: &Export, dll_name: &str, types: &TypeMap) -> TokenStream {
    match export {
        Export::Fn(export) => {
            let dll_import_attrib = quote_dll_import(dll_name, &export.binding);
            let binding_ident = format_ident!("{}", &*export.binding);
            let return_ty = match &export.output {
                Some(output) => quote_type_binding(output, types),
                None => quote! { void },
            };
            let args = quote_binding_args(export.inputs(), types);

            quote! {
                #dll_import_attrib
                internal static extern #return_ty #binding_ident(#args);
            }
        }

        Export::Method(export) => {
            let dll_import_attrib = quote_dll_import(dll_name, &export.binding);
            let binding_ident = format_ident!("{}", &*export.binding);
            let return_ty = match &export.output {
                Some(output) => quote_type_binding(output, types),
                None => quote! { void },
            };

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

        // For named types, we generate some additional bindings:
        //
        // * For handle types, the Rust module exports a drop function that we need to free
        //   the memory for the object.
        // * We generate from-raw and into-raw conversion functions in order to convert the
        //   C# representation of the type to-and-from the raw representation that can be
        //   passed to Rust.
        Export::Named(export) => {
            // Generate the drop function for the type (if needed).
            let drop_fn = if export.binding_style == BindingStyle::Handle {
                class::quote_drop_fn(&export.name, dll_name)
            } else {
                quote! {}
            };

            // Generate the raw conversion functions for the type.
            let from_raw = from_raw_ident(&export.name);
            let into_raw = into_raw_ident(&export.name);

            let raw_fns = match &export.schema {
                // TODO: Support raw conversions for structs, too!
                Schema::Struct(_) => quote! {},

                Schema::Enum(schema) => {
                    let raw_repr = enumeration::quote_raw_type_reference(export, schema);
                    let cs_repr = enumeration::quote_type_reference(export, schema);

                    quote! {
                        #drop_fn

                        internal static #cs_repr #from_raw(#raw_repr raw)
                        {
                            // TODO
                            throw new NotImplementedException();
                        }

                        internal static #raw_repr #into_raw(#cs_repr self)
                        {
                            // TODO
                            throw new NotImplementedException();
                        }
                    }
                }

                _ => todo!("What would we even do here???"),
            };

            quote! {
                #drop_fn
                #raw_fns
            }
        }
    }
}

pub fn quote_type_binding(schema: &Schema, types: &TypeMap) -> TokenStream {
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

        Schema::String => quote! { RustOwnedString },

        Schema::Enum(schema) => quote_enum_binding(schema, types),

        Schema::Struct(schema) => quote_struct_binding(schema, types),

        // TODO: Add support for passing user-defined types to Rust.
        Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_)
        | Schema::Option(_)
        | Schema::Seq(_)
        | Schema::Tuple(_)
        | Schema::Map { .. } => todo!("Generate argument binding"),

        Schema::Unit => quote! { byte },

        Schema::I128 | Schema::U128 => {
            unreachable!("Invalid types should have already been handled")
        }
    }
}

fn quote_struct_binding(schema: &Struct, types: &TypeMap) -> TokenStream {
    let export = types
        .get(&schema.name)
        .expect("Couldn't find exported type for struct");

    match export.binding_style {
        BindingStyle::Handle => quote! { void* },
        BindingStyle::Value => todo!("Support passing structs by value"),
    }
}

fn quote_enum_binding(schema: &Enum, types: &TypeMap) -> TokenStream {
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
        //   `isize` (`IntPtr`).
        BindingStyle::Value => {
            if schema.has_data() {
                let raw = format_ident!("{}__Raw", &*export.name);
                quote! { RawEnum<#raw> }
            } else {
                enumeration::quote_discriminant_type(schema)
            }
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
            let ty = quote_type_binding(schema, types);

            quote! { #ty #ident }
        })
        .collect()
}
