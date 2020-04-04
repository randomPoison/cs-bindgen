//! Code generation for exported struct types.

use crate::generate::{self, binding, class, TypeMap};
use cs_bindgen_shared::{
    schematic::{Field, StructLike},
    BindingStyle, NamedType,
};
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::*;
use syn::Ident;

pub fn quote_struct(export: &NamedType, schema: StructLike<'_>, types: &TypeMap) -> TokenStream {
    if export.binding_style == BindingStyle::Handle {
        return class::quote_handle_type(export);
    }

    let ident = format_ident!("{}", &*export.name);
    let raw_ident = binding::raw_ident(&export.name);

    let field_ident = schema
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| field_ident(field.name, index))
        .collect::<Vec<_>>();

    let struct_fields = struct_fields(&schema.fields, types);
    let basic_constructor = struct_constructor(&ident, &schema.fields, types);
    let raw_fields = binding::raw_struct_fields(&schema.fields, types);

    let bindings = binding::bindings_class_ident();
    let from_raw = binding::from_raw_fn_ident();
    let into_raw = binding::into_raw_fn_ident();

    let raw_conversions = binding::wrap_bindings(quote! {
        internal static #ident #from_raw(#raw_ident raw)
        {
            return new #ident(raw);
        }

        internal static #raw_ident #into_raw(#ident self)
        {
            return new #raw_ident(self);
        }
    });

    quote! {
        public struct #ident
        {
            #struct_fields
            #basic_constructor

            // Generate a constructor that can initialize the struct from its raw version.
            internal #ident(#raw_ident raw)
            {
                #(
                    this.#field_ident = #bindings.#from_raw(raw.#field_ident);
                )*
            }
        }

        internal struct #raw_ident
        {
            #raw_fields

            internal #raw_ident(#ident self)
            {
                #(
                    this.#field_ident = #bindings.#into_raw(self.#field_ident);
                )*
            }
        }

        #raw_conversions
    }
}

/// Quotes the field declarations for the generated C# struct corresponding to an
/// exported Rust type.
pub fn struct_fields(fields: &[Field<'_>], types: &TypeMap) -> TokenStream {
    let field_ident = fields
        .iter()
        .enumerate()
        .map(|(index, field)| field_ident(field.name, index));

    let field_ty = fields
        .iter()
        .map(|field| generate::quote_cs_type(&field.schema, types));

    quote! {
        #(
            public #field_ty #field_ident;
        )*
    }
}

/// Quotes the basic constructor for the given type.
///
/// The basic constructor has a parameter for each field in the struct, and directly
/// assigns each field.
pub fn struct_constructor(ident: &Ident, fields: &[Field<'_>], types: &TypeMap) -> TokenStream {
    let field_ident = fields
        .iter()
        .enumerate()
        .map(|(index, field)| field_ident(field.name, index));

    let arg_ident = fields
        .iter()
        .enumerate()
        .map(|(index, field)| arg_ident(field.name, index))
        .collect::<Vec<_>>();

    let field_ty = fields
        .iter()
        .map(|field| generate::quote_cs_type(&field.schema, types));

    quote! {
        public #ident(#( #field_ty #arg_ident ),*)
        {
            #(
                this.#field_ident = #arg_ident;
            )*
        }
    }
}

/// Converts the specified field name into a C#-appropriate ident, or generates an
/// ident based on the index of the field if the field is unnamed.
pub fn field_ident(name: Option<&str>, index: usize) -> Ident {
    name.map(|name| format_ident!("{}", name.to_camel_case()))
        .unwrap_or_else(|| format_ident!("Element{}", index))
}

fn arg_ident(name: Option<&str>, index: usize) -> Ident {
    name.map(|name| format_ident!("{}", name))
        .unwrap_or_else(|| format_ident!("element_{}", index))
}
