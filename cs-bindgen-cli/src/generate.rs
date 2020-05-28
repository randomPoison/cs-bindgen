use self::{binding::*, class::*, enumeration::*, func::*};
use crate::Opt;
use cs_bindgen_shared::{
    schematic::{self, Primitive, Schema, TypeName},
    BindingStyle, Export, NamedType, Repr,
};
use heck::*;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::*;
use std::{collections::HashMap, ffi::OsStr};
use syn::Ident;

mod binding;
mod class;
mod enumeration;
mod func;
mod strukt;

type TypeMap<'a> = HashMap<&'a TypeName, &'a NamedType>;

lazy_static! {
    static ref STRING_SCHEMA: Schema = schematic::describe::<String>();
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
            Export::Named(export) => Some((&export.type_name, export)),
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
                &export.inputs,
                export.output.as_ref(),
                &types,
            )),

            Export::Named(export) => match &export.binding_style {
                BindingStyle::Handle => binding_items.push(class::quote_handle_type(export)),

                BindingStyle::Value(schema) => match schema {
                    Schema::Struct(_)
                    | Schema::TupleStruct(_)
                    | Schema::UnitStruct(_)
                    | Schema::NewtypeStruct(_) => binding_items.push(strukt::quote_struct(
                        export,
                        // NOTE: The unwrap here will not panic because all of the matched variants have
                        // a struct-like representation. If it panics here, then it likely indicates a
                        // bug in the schematic crate.
                        schema.as_struct_like().unwrap(),
                        &types,
                    )),

                    Schema::Enum(schema) => binding_items.push(quote_enum(export, schema, &types)),

                    _ => {
                        return Err(failure::format_err!(
                            "Invalid schema for exported type {:?}: {:?}",
                            export.type_name,
                            schema
                        ))
                    }
                },
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
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_u8(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_i8(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_u16(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_i16(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_u32(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_i32(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_u64(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_i64(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_usize(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_isize(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_f32(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_f64(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_bool(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void __cs_bindgen_drop_vec_char(RawVec raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_u8(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_i8(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_u16(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_i16(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_u32(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_i32(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_u64(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_i64(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_usize(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_isize(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_f32(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_f64(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_bool(RawSlice raw);

        [DllImport(
            #dll_name,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern RawVec __cs_bindgen_convert_vec_char(RawSlice raw);

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
            result = raw != 0;
        }

        internal static void __FromRaw(RawVec raw, out string result)
        {
            result = Encoding.UTF8.GetString((byte*)raw.Ptr, (int)raw.Length);
            __bindings.__cs_bindgen_drop_vec_u8(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<byte> result)
        {
            result = raw.ToPrimitiveList<byte>();
            __bindings.__cs_bindgen_drop_vec_u8(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<sbyte> result)
        {
            result = raw.ToPrimitiveList<sbyte>();
            __bindings.__cs_bindgen_drop_vec_i8(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<short> result)
        {
            result = raw.ToPrimitiveList<short>();
            __bindings.__cs_bindgen_drop_vec_i16(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<ushort> result)
        {
            result = raw.ToPrimitiveList<ushort>();
            __bindings.__cs_bindgen_drop_vec_u16(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<int> result)
        {
            result = raw.ToPrimitiveList<int>();
            __bindings.__cs_bindgen_drop_vec_i32(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<uint> result)
        {
            result = raw.ToPrimitiveList<uint>();
            __bindings.__cs_bindgen_drop_vec_u32(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<long> result)
        {
            result = raw.ToPrimitiveList<long>();
            __bindings.__cs_bindgen_drop_vec_i64(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<ulong> result)
        {
            result = raw.ToPrimitiveList<ulong>();
            __bindings.__cs_bindgen_drop_vec_u64(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<float> result)
        {
            result = raw.ToPrimitiveList<float>();
            __bindings.__cs_bindgen_drop_vec_f32(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<double> result)
        {
            result = raw.ToPrimitiveList<double>();
            __bindings.__cs_bindgen_drop_vec_f64(raw);
        }

        internal static void __FromRaw(RawVec raw, out List<bool> result)
        {
            result = raw.ToPrimitiveList<byte, bool>(rawElem => rawElem != 0);
            __bindings.__cs_bindgen_drop_vec_u8(raw);
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
                result = __cs_bindgen_string_from_utf16(new RawSlice((IntPtr)charPtr, value.Length));
            }
        }

        internal static void __IntoRaw(List<byte> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_u8);
        }

        internal static void __IntoRaw(List<sbyte> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_i8);
        }

        internal static void __IntoRaw(List<short> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_i16);
        }

        internal static void __IntoRaw(List<ushort> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_u16);
        }

        internal static void __IntoRaw(List<int> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_i32);
        }

        internal static void __IntoRaw(List<uint> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_u32);
        }

        internal static void __IntoRaw(List<long> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_i64);
        }

        internal static void __IntoRaw(List<ulong> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_u64);
        }

        internal static void __IntoRaw(List<float> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_f32);
        }

        internal static void __IntoRaw(List<double> value, out RawVec result)
        {
            result = RawVec.FromPrimitiveList(value, __cs_bindgen_convert_vec_f64);
        }

        internal static void __IntoRaw(List<bool> value, out RawVec result)
        {
            result = RawVec.FromList<bool, byte>(
                value,
                item => item ? (byte)1 : (byte)0,
                __cs_bindgen_convert_vec_bool);
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

        internal delegate void FromRaw<R, T>(R raw, out T result);

        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct RawVec
        {
            public IntPtr Ptr;
            public UIntPtr Length;
            public UIntPtr Capacity;

            public List<T> ToPrimitiveList<T>() where T: unmanaged
            {
                var result = new List<T>((int)Length);
                var orig = (T*)Ptr;

                for (int index = 0; index < (int)Length; index += 1)
                {
                    result.Add(orig[index]);
                }

                return result;
            }

            public List<T> ToPrimitiveList<D, T>(Func<D, T> conversion) where D: unmanaged
            {
                var result = new List<T>((int)Length);
                var orig = (D*)Ptr;

                for (int index = 0; index < (int)Length; index += 1)
                {
                    result.Add(conversion(orig[index]));
                }

                return result;
            }

            public List<T> ToList<R, T>(
                Func<RawSlice, UIntPtr, R> indexFn,
                FromRaw<R, T> fromRaw)
            where R: unmanaged
            {
                var slice = AsSlice();
                var result = new List<T>((int)Length);

                for (int index = 0; index < (int)Length; index += 1)
                {
                    R rawElement = indexFn(slice, (UIntPtr)index);
                    fromRaw(rawElement, out T element);
                    result.Add(element);
                }

                return result;
            }

            public static RawVec FromPrimitiveList<T>(List<T> items, Func<RawSlice, RawVec> allocVec)
                where T: unmanaged
            {
                // TODO: It would be nice to not have to copy the list in order to get the pointer.
                // Support for getting a `Span<T>` from a `List<T>` is supposedly coming in
                // netstandard5.0, though even then we wouldn't be able to use it in Unity for a
                // while.
                var array = items.ToArray();
                fixed (T* ptr = array)
                {
                    return allocVec(new RawSlice((IntPtr)ptr, items.Count));
                }
            }

            public static RawVec FromList<T, R>(List<T> items, Func<T, R> convertElement, Func<RawSlice, RawVec> handleResult)
                where R : unmanaged
            {
                // If the list is small enough, allocate the temporary list of raw items on the
                // stack to avoid unnecessary heap allocation. We use 32 as a fairly arbitrary
                // cutoff, with the hope that it's small enough to be unlikely to overflow the
                // stack.
                if (items.Count <= 32)
                {
                    R* rawItems = stackalloc R[items.Count];

                    for (int index = 0; index < items.Count; index += 1)
                    {
                        rawItems[index] = convertElement(items[index]);
                    }

                    return handleResult(new RawSlice((IntPtr)rawItems, items.Count));
                }
                else
                {
                    var rawItems = new R[items.Count];
                    for (int index = 0; index < items.Count; index += 1)
                    {
                        rawItems[index] = convertElement(items[index]);
                    }

                    fixed (R* ptr = rawItems)
                    {
                        return handleResult(new RawSlice((IntPtr)ptr, items.Count));
                    }
                }
            }

            public RawSlice AsSlice()
            {
                return new RawSlice(Ptr, Length);
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct RawSlice
        {
            public IntPtr Ptr;
            public UIntPtr Length;

            public RawSlice(IntPtr ptr, UIntPtr len)
            {
                Ptr = ptr;
                Length = len;
            }

            public RawSlice(IntPtr ptr, int len)
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

fn quote_cs_type_for_repr(repr: &Repr, types: &TypeMap) -> TokenStream {
    let quote_sequence_type = |element| {
        let element = quote_cs_type_for_repr(element, types);
        quote! {
            List<#element>
        }
    };

    match repr {
        Repr::Unit => todo!("Support unit types"),

        Repr::Bool => quote! { bool },

        Repr::Char => todo!("Support passing `char` values"),

        Repr::I8 => quote! { sbyte },
        Repr::I16 => quote! { short },
        Repr::I32 => quote! { int },
        Repr::I64 => quote! { long },
        Repr::ISize => quote! { IntPtr },

        Repr::U8 => quote! { byte },
        Repr::U16 => quote! { ushort },
        Repr::U32 => quote! { uint },
        Repr::U64 => quote! { ulong },
        Repr::USize => quote! { UIntPtr },

        Repr::F32 => quote! { float },
        Repr::F64 => quote! { double },

        Repr::Named(type_name) => {
            let export = types
                .get(type_name)
                .unwrap_or_else(|| panic!("Could not resolve type reference: {:?}", type_name));

            // NOTE: Enums that are exported by value are a special case since the user-facing
            // type for a data-carrying enum is an interface, and therefore has a different
            // naming convention from Rust structs.
            let ident = match &export.binding_style {
                BindingStyle::Value(Schema::Enum(schema)) => {
                    enumeration::quote_type_reference(schema)
                }
                _ => format_ident!("{}", &*export.type_name.name).into_token_stream(),
            };

            // TODO: Take into account things like custom namespaces or renaming the type, once
            // those are supported. For now, we manually prefix references to user-defined types
            // with `global::` in order to avoid name collisions. Once we support custom
            // namespaces, we'll want to use the correct namespace name instead.
            quote! { global::#ident }
        }

        Repr::Vec(inner) => quote_sequence_type(inner),
        Repr::Slice(inner) => quote_sequence_type(inner),
        Repr::Array { element, .. } => quote_sequence_type(element),

        Repr::String | Repr::Str => quote! { string },

        Repr::Option(_) => todo!("Support optional values"),
        Repr::Result { .. } => todo!("Support results"),

        Repr::Box(_) | Repr::Ref(_) => todo!("Support pointer types"),
    }
}

/// Generates the idiomatic C# type corresponding to the given type schema.
fn quote_cs_type_for_schema(schema: &Schema, types: &TypeMap) -> TokenStream {
    let quote_sequence_type = |element| {
        let element = quote_cs_type_for_schema(element, types);
        quote! {
            List<#element>
        }
    };

    // Create a helper closure for generating references to named types in a uniform way.
    let named_type_reference = |type_name, types: &TypeMap| {
        let export = types
            .get(type_name)
            .unwrap_or_else(|| panic!("Could not resolve type reference: {:?}", type_name));

        // NOTE: Enums that are exported by value are a special case since the user-facing
        // type for a data-carrying enum is an interface, and therefore has a different
        // naming convention from Rust structs.
        let ident = match &export.binding_style {
            BindingStyle::Value(Schema::Enum(schema)) => enumeration::quote_type_reference(schema),
            _ => format_ident!("{}", &*export.type_name.name).into_token_stream(),
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
            let key = quote_cs_type_for_schema(&schema.key, types);
            let value = quote_cs_type_for_schema(&schema.value, types);
            quote! {
                Dictionary<#key, #value>
            }
        }

        // Generate an unnamed tuple type for an exported Rust tuple. Conveniently this has
        // the same syntax in C# as in Rust, e.g. `(int, Foo, Bar)`. How nice!
        Schema::Tuple(elements) => {
            let element = elements
                .iter()
                .map(|schema| quote_cs_type_for_schema(schema, types));
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

#[extend::ext]
impl TypeName {
    fn ident(&self) -> Ident {
        format_ident!("{}", self.name)
    }
}
