use self::{binding::*, class::*, enumeration::*, func::*};
use crate::Opt;
use cs_bindgen_shared::{
    schematic::{Primitive, Schema, TypeName},
    Export, NamedType,
};
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use std::{collections::HashMap, ffi::OsStr};

mod binding;
mod class;
mod enumeration;
mod func;

type TypeMap<'a> = HashMap<&'a TypeName, &'a NamedType>;

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

    // Gather the definitions for all user-defined types so that the full export
    // information can be retrieved when an export represents another exported type.
    let type_map = exports
        .iter()
        .filter_map(|export| match export {
            Export::Named(export) => Some((
                export
                    .schema
                    .type_name()
                    .expect("Named type's schema did not have a type name"),
                export,
            )),
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    // Generate the raw bindings for all exported items.
    let raw_bindings = exports
        .iter()
        .map(|item| quote_raw_binding(item, dll_name))
        .collect::<Vec<_>>();

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
                &type_map,
            )),

            Export::Named(export) => match &export.schema {
                Schema::Struct(schema) => binding_items.push(quote_struct(export, schema)),
                Schema::Enum(schema) => {
                    binding_items.push(quote_enum_binding(export, schema, &type_map))
                }

                _ => {
                    return Err(failure::format_err!(
                        "Invalid schema for exported type {}: {:?}",
                        export.name,
                        export.schema
                    ))
                }
            },

            Export::Method(export) => binding_items.push(quote_method_binding(export, &type_map)),
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

        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct RawEnum<D, V>
            where D : unmanaged
            where V : unmanaged
        {
            public D Discriminant;
            public V Value;
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

/// Generates the idiomatic C# type corresponding to the given type schema.
fn quote_cs_type(schema: &Schema, type_map: &TypeMap) -> TokenStream {
    match schema {
        // NOTE: This is only valid in a return position, it's not valid to have a `void`
        // argument. An earlier validation pass has already rejected any such cases so we
        // don't have to differentiate between the two here.
        Schema::Unit => quote! { void },

        // TODO: Should we be generating more idiomatic return types for numeric types? Any
        // numeric type that's not `int`, `long`, or `float`/double` is going to be awkward
        // to use in most cases.
        //
        // Tracking issue: https://github.com/randomPoison/cs-bindgen/issues/4
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
        Schema::Bool => quote! { bool },
        Schema::String => quote! { string },

        Schema::Char => todo!("Support passing single chars"),

        Schema::Struct(schema) => {
            let export = type_map
                .get(&schema.name)
                .expect("Failed to look up referenced type");

            // TODO: Take into account things like custom namespaces or renaming the type, once
            // those are supported.
            format_ident!("{}", &*export.name).into_token_stream()
        }

        Schema::Enum(schema) => {
            let export = type_map
                .get(&schema.name)
                .expect("Failed to look up referenced type");
            enumeration::quote_type_reference(&export, schema)
        }

        // TODO: Add support for passing user-defined types out from Rust.
        Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_)
        | Schema::Option(_)
        | Schema::Seq(_)
        | Schema::Tuple(_)
        | Schema::Map { .. } => todo!("Generate argument binding"),

        Schema::I128 | Schema::U128 => {
            unreachable!("Invalid argument types should have already been rejected");
        }
    }
}
