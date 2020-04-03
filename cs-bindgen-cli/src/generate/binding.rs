//! Utilities for generating the raw bindings to exported Rust items.
//!
//! In C#, the raw binding to an exported Rust function is a `static extern`
//! function, using the `[DllImport]` attribute to load the corresponding function
//! from the Rust dylib. This module provides

use crate::generate::{class, enumeration, quote_cs_type, TypeMap};
use cs_bindgen_shared::{schematic::Schema, BindingStyle, Export};
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma, Ident};

// TODO: For the below functions that generate identifiers based on a type name, we
// should use the fully-qualified `TypeName` instead of just a `&str` name. Right
// now, if two types with the same name in different modules are exported, the
// generated bindings will collide. We can avoid this by taking the module name into
// account when generating the idents. This will require some additional mangling
// logic, since the module paths include `::` characters, which aren't valid in C#
// identifiers.

/// The identifier of the from-raw conversion method.
///
/// This method is overloaded for every supported primitive and exported type, so it
/// can be used as a generic way to perform type conversion.
pub fn from_raw_fn_ident() -> Ident {
    format_ident!("__FromRaw")
}

/// The identifier of the into-raw conversion method.
///
/// This method is overloaded for every supported primitive and exported type, so it
/// can be used as a generic way to perform type conversion.
pub fn into_raw_fn_ident() -> Ident {
    format_ident!("__IntoRaw")
}

/// Generate the identifier for the raw type corresponding to the specified type.
///
/// When a user-defined type is marshaled by value, we generate a type that acts as
/// an FFI-safe "raw" representation for that type. When communicating with Rust, we
/// convert the C# representation of the type to-and-from the raw representation.
/// This function provides the canonical way to generate the name of the raw type
/// corresponding to any given exported Rust type.
pub fn raw_ident(name: &str) -> Ident {
    format_ident!("__{}__Raw", name)
}

pub fn quote_raw_binding(export: &Export, dll_name: &str, types: &TypeMap) -> TokenStream {
    match export {
        Export::Fn(export) => {
            let dll_import_attrib = quote_dll_import(dll_name, &export.binding);
            let binding_ident = format_ident!("{}", &*export.binding);
            let return_ty = match &export.output {
                Some(output) => quote_raw_type_reference(output, types),
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
                Some(output) => quote_raw_type_reference(output, types),
                None => quote! { void },
            };

            // TODO: Unify input handling for raw bindings. It shouldn't be necessary to
            // manually insert the receiver. The current blocker is that schematic can't
            // represent reference types, so we can't generate a full list of inputs that
            // includes the receiver.
            let mut args = quote_binding_args(export.inputs(), types);
            if export.receiver.is_some() {
                let handle_type = class::quote_handle_ptr();
                args.insert(0, quote! { #handle_type self });
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

            let from_raw = from_raw_fn_ident();
            let into_raw = into_raw_fn_ident();

            let cs_repr = quote_cs_type(&export.schema, types);
            let raw_repr = quote_raw_type_reference(&export.schema, types);

            let (from_raw_impl, into_raw_impl) = match export.binding_style {
                BindingStyle::Handle => {
                    let from_raw_impl = quote! {
                        return new #cs_repr(raw);
                    };
                    let into_raw_impl = quote! {
                        return new #raw_repr(self);
                    };

                    (from_raw_impl, into_raw_impl)
                }

                BindingStyle::Value => match &export.schema {
                    Schema::Struct(_) => {
                        let from_raw_impl = quote! {
                            throw new NotImplementedException("Support passing structs by value");
                        };
                        let into_raw_impl = quote! {
                            throw new NotImplementedException("Support passing structs by value");
                        };

                        (from_raw_impl, into_raw_impl)
                    }

                    Schema::Enum(schema) => {
                        let from_raw_impl = enumeration::from_raw_impl(export, schema);
                        let into_raw_impl = enumeration::into_raw_impl(export, schema);

                        (from_raw_impl, into_raw_impl)
                    }

                    Schema::UnitStruct(..)
                    | Schema::TupleStruct(..)
                    | Schema::NewtypeStruct(..) => {
                        todo!("Support more kinds of user-defined types")
                    }

                    _ => unreachable!("Named type had invalid schema: {:?}", export.schema),
                },
            };

            quote! {
                #drop_fn

                internal static #cs_repr #from_raw(#raw_repr raw)
                {
                    #from_raw_impl
                }

                internal static #raw_repr #into_raw(#cs_repr self)
                {
                    #into_raw_impl
                }
            }
        }
    }
}

/// Generates the appropriate raw type name for the given type schema.
pub fn quote_raw_type_reference(schema: &Schema, types: &TypeMap) -> TokenStream {
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
        Schema::Bool => quote! { RustBool },
        Schema::Char => quote! { uint },
        Schema::String => quote! { RustOwnedString },

        Schema::Enum(schema) => {
            let ident = raw_ident(&schema.name.name);
            quote! {
                global::#ident
            }
        }

        Schema::Struct(schema) => {
            let ident = raw_ident(&schema.name.name);
            quote! {
                global::#ident
            }
        }

        // TODO: Add support for more user-defined types.
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
            let ty = quote_raw_type_reference(schema, types);

            quote! { #ty #ident }
        })
        .collect()
}
