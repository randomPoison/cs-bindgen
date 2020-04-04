//! Utilities for generating the raw bindings to exported Rust items.
//!
//! This module provides the code generation for the C# declarations that bind to
//! Rust functions that are exported from the built dylib. Note that this
//! specifically refers to the *generated* functions, not the user defined
//! functions. This module also provides utilities for referencing the raw function
//! bindings in other parts of the code generation.
//!
//! In C#, the raw binding to an exported Rust function is a `static extern`
//! function, using the `[DllImport]` attribute to load the corresponding function
//! from the Rust dylib.

use crate::generate::{class, strukt, TypeMap};
use cs_bindgen_shared::{
    schematic::{Field, Schema, TypeName},
    BindingStyle, Export,
};
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

/// Returns the identifier of the generating bindings class.
pub fn bindings_class_ident() -> Ident {
    format_ident!("__bindings")
}

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

pub fn wrap_bindings(tokens: TokenStream) -> TokenStream {
    quote! {
        internal unsafe static partial class __bindings
        {
            #tokens
        }
    }
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

        // Generate the binding for the destructor for any named types that are marshaled
        // as handles.
        Export::Named(export) => {
            if export.binding_style == BindingStyle::Handle {
                class::quote_drop_fn(&export, dll_name)
            } else {
                quote! {}
            }
        }
    }
}

/// Generates the appropriate raw type name for the given type schema.
// NOTE: We're not currently using the type map parameter, but we'll eventually need
// it once we support custom namespaces, since we'll need to look up the export
// information to determine the fully-qualified name for the type.
pub fn quote_raw_type_reference(schema: &Schema, _types: &TypeMap) -> TokenStream {
    fn named_type_raw_reference(type_name: &TypeName) -> TokenStream {
        let ident = raw_ident(&type_name.name);
        quote! {
            global::#ident
        }
    }

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

        // NOTE: The unwrap here is valid because all of the struct-like variants are
        // guaranteed to have a type name. If this panic, that indicates a bug in the
        // schematic crate.
        Schema::Enum(_)
        | Schema::Struct(_)
        | Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_) => named_type_raw_reference(schema.type_name().unwrap()),

        // TODO: Add support for collection types.
        Schema::Option(_) | Schema::Seq(_) | Schema::Tuple(_) | Schema::Map { .. } => {
            todo!("Generate argument binding")
        }

        Schema::Unit => quote! { byte },

        Schema::I128 | Schema::U128 => {
            unreachable!("Invalid types should have already been handled")
        }
    }
}

/// Generates the field definitions for the raw struct representation of an exported
/// Rust type.
pub fn raw_struct_fields(fields: &[Field<'_>], types: &TypeMap) -> TokenStream {
    let field_name = fields
        .iter()
        .enumerate()
        .map(|(index, field)| strukt::field_ident(field.name, index));

    let field_ty = fields
        .iter()
        .map(|field| quote_raw_type_reference(&field.schema, types));

    quote! {
        #(
            internal #field_ty #field_name;
        )*
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
