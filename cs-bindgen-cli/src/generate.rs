use self::{binding::*, class::*, enumeration::*, func::*};
use crate::Opt;
use cs_bindgen_shared::{
    schematic::{self, Primitive, Schema, TypeName},
    Export, NamedType,
};
use heck::*;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::*;
use std::{collections::HashMap, ffi::OsStr};

mod binding;
mod class;
mod enumeration;
mod func;
mod strukt;

type TypeMap<'a> = HashMap<&'a TypeName, &'a NamedType>;

lazy_static! {
    static ref STRING_SCHEMA: Schema = schematic::describe::<String>().unwrap();
}

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
    let types = exports
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
        .map(|item| quote_raw_binding(item, dll_name, &types))
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
                export.output.as_ref(),
                &types,
            )),

            Export::Named(export) => match &export.schema {
                Schema::Struct(_)
                | Schema::TupleStruct(_)
                | Schema::UnitStruct(_)
                | Schema::NewtypeStruct(_) => binding_items.push(strukt::quote_struct(
                    export,
                    // NOTE: The unwrap here will not panic because all of the matched variants have
                    // a struct-like representation. If it panics here, then it likely indicates a
                    // bug in the schematic crate.
                    export.schema.as_struct_like().unwrap(),
                    &types,
                )),

                Schema::Enum(schema) => binding_items.push(quote_enum(export, schema, &types)),

                _ => {
                    return Err(failure::format_err!(
                        "Invalid schema for exported type {}: {:?}",
                        export.name,
                        export.schema
                    ))
                }
            },

            Export::Method(export) => binding_items.push(quote_method_binding(export, &types)),
        }
    }

    // Wrap the raw bindings for exported functions/methods in the bindings class definition.
    let raw_bindings = binding::wrap_bindings(quote! {
        #( #raw_bindings )*
    });

    let built_in_bindings = binding::wrap_bindings(quote! {
        // Bindings to built-in helper functions.
        [DllImport(
            #dll_name,
            EntryPoint = "__cs_bindgen_drop_string",
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_string(RawVec raw);

        [DllImport(
            #dll_name,
            EntryPoint = "__cs_bindgen_string_from_utf16",
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_string_from_utf16(RawSlice raw);

        // Overloads of `__FromRaw` for primitives and built-in types.
        internal static void __FromRaw(byte raw, out byte result) { result = raw; }
        internal static void __FromRaw(sbyte raw, out sbyte result) { result = raw; }
        internal static void __FromRaw(short raw, out short result) { result = raw; }
        internal static void __FromRaw(ushort raw, out ushort result) { result = raw; }
        internal static void __FromRaw(int raw, out int result) { result = raw; }
        internal static void __FromRaw(uint raw, out uint result) { result = raw; }
        internal static void __FromRaw(long raw, out long result) { result = raw; }
        internal static void __FromRaw(ulong raw, out ulong result) { result = raw; }
        internal static void __FromRaw(float raw, out float result) { result = raw; }
        internal static void __FromRaw(double raw, out double result) { result = raw; }

        internal static void __FromRaw(byte raw, out bool result)
        {
            result = raw != 0 ? true : false;
        }

        internal static void __FromRaw(RawVec raw, out string result)
        {
            result = Encoding.UTF8.GetString((byte*)raw.Ptr, (int)raw.Length);
            __bindings.__cs_bindgen_drop_string(raw);
        }

        internal static void __FromRaw<T>(RawVec raw, out List<T> result)
        {
            var output = new List<T>();
            for (int index = 0; index < raw.Length; index += 1)
            {
                // Heck how do we index into the array? Don't we need to know the size/alignment of the element type?
            }
        }

        // Overloads of `__IntoRaw` for primitives and built-in types.
        internal static void __IntoRaw(byte value, out byte result) { result = value; }
        internal static void __IntoRaw(sbyte value, out sbyte result) { result = value; }
        internal static void __IntoRaw(short value, out short result) { result = value; }
        internal static void __IntoRaw(ushort value, out ushort result) { result = value; }
        internal static void __IntoRaw(int value, out int result) { result = value; }
        internal static void __IntoRaw(uint value, out uint result) { result = value; }
        internal static void __IntoRaw(long value, out long result) { result = value; }
        internal static void __IntoRaw(ulong value, out ulong result) { result = value; }
        internal static void __IntoRaw(float value, out float result) { result = value; }
        internal static void __IntoRaw(double value, out double result) { result = value; }

        internal static void __IntoRaw(bool value, out byte result)
        {
            result = value ? (byte)1 : (byte)0;
        }

        internal static void __IntoRaw(string value, out RawVec result)
        {
            fixed (char* charPtr = value)
            {
                result = __cs_bindgen_string_from_utf16(new RawSlice((void*)charPtr, value.Length));
            }
        }
    });

    let generated = quote! {
        using System;
        using System.Collections.Generic;
        using System.Runtime.InteropServices;
        using System.Text;

        #built_in_bindings
        #raw_bindings

        public class #class_name
        {
            #( #fn_bindings )*
        }

        #( #binding_items )*

        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct RawVec
        {
            public void* Ptr;
            public UIntPtr Length;
            public UIntPtr Capacity;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct RawSlice
        {
            public void* Ptr;
            public UIntPtr Length;

            public RawSlice(void* ptr, UIntPtr len)
            {
                Ptr = ptr;
                Length = len;
            }

            public RawSlice(void* ptr, int len)
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

/// Generates the idiomatic C# type corresponding to the given type schema.
fn quote_cs_type(schema: &Schema, types: &TypeMap) -> TokenStream {
    let quote_sequence_type = |element| {
        let element = quote_cs_type(element, types);
        quote! {
            List<#element>
        }
    };

    // Create a helper closure for generating references to named types in a uniform way.
    let named_type_reference = |type_name, types: &TypeMap| {
        let export = types
            .get(type_name)
            .unwrap_or_else(|| panic!("Could not resolve type reference: {:?}", type_name));

        // NOTE: Enums are a special case since the user-facing type for a data-carrying
        // enum is an interface, and therefore has a different naming convention from
        // Rust structs.
        let ident = if let Schema::Enum(schema) = &schema {
            enumeration::quote_type_reference(export, schema)
        } else {
            format_ident!("{}", &*export.name).into_token_stream()
        };

        // TODO: Take into account things like custom namespaces or renaming the type, once
        // those are supported. For now, we manually prefix references to user-defined types
        // with `global::` in order to avoid name collisions. Once we support custom
        // namespaces, we'll want to use the correct namespace name instead.
        quote! { global::#ident }
    };

    match schema {
        // NOTE: This is only valid in a return position, it's not valid to have a `void`
        // argument. An earlier validation pass has already rejected any such cases so we
        // don't have to differentiate between the two here.
        Schema::Unit => quote! { void },

        Schema::Bool => quote! { bool },

        // TODO: Should we be generating more idiomatic return types for numeric types? Any
        // numeric type that's not `int`, `long`, or `float`/double` is going to be awkward
        // to use in most cases.
        //
        // Tracking issue: https://github.com/randomPoison/cs-bindgen/issues/4
        Schema::I8 => quote! { sbyte },
        Schema::I16 => quote! { short },
        Schema::I32 => quote! { int },
        Schema::I64 => quote! { long },
        Schema::ISize => quote! { IntPtr },

        Schema::U8 => quote! { byte },
        Schema::U16 => quote! { ushort },
        Schema::U32 => quote! { uint },
        Schema::U64 => quote! { ulong },
        Schema::USize => quote! { UIntPtr },

        Schema::F32 => quote! { float },
        Schema::F64 => quote! { double },

        Schema::Char => todo!("Support passing single chars"),

        Schema::Str | Schema::String(_) => quote! { string },

        // NOTE: The unwrap here is valid because all of the struct-like variants are
        // guaranteed to have a type name. If this panics, that indicates a bug in the
        // schematic crate.
        Schema::Enum(_)
        | Schema::Struct(_)
        | Schema::TupleStruct(_)
        | Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_) => named_type_reference(schema.type_name().unwrap(), types),

        // All sequence types are exposed in C# as a `List<T>`, since for all practical
        // purposes that's the most efficient and flexible option.
        Schema::Array(schema) => quote_sequence_type(&schema.element),
        Schema::Slice(element) => quote_sequence_type(element),
        Schema::Seq(schema) => quote_sequence_type(&schema.element),

        // Map types are exposed in C# as a `Dictionary<K, V>`.
        Schema::Map(schema) => {
            let key = quote_cs_type(&schema.key, types);
            let value = quote_cs_type(&schema.value, types);
            quote! {
                Dictionary<#key, #value>
            }
        }

        // Generate an unnamed tuple type for an exported Rust tuple. Conveniently this has
        // the same syntax in C# as in Rust, e.g. `(int, Foo, Bar)`. How nice!
        Schema::Tuple(elements) => {
            let element = elements.iter().map(|schema| quote_cs_type(schema, types));
            quote! {
                ( #( #element ),* )
            }
        }

        // TODO: Add support for optional types. In order to do so, we'll need to determine
        // if the type is a reference type or a value type. Reference types are inherently
        // nullable, but value types need to be converted to `Nullable<T>` (or `T?` for
        // short).
        Schema::Option(_) => todo!("Generate nullable type reference"),

        Schema::I128 | Schema::U128 => {
            unreachable!("Invalid argument types should have already been rejected");
        }
    }
}
