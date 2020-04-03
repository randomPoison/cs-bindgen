use crate::generate::{self, binding, class, TypeMap};
use cs_bindgen_shared::{
    schematic::{Schema, Struct},
    BindingStyle, NamedType,
};
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::*;
use syn::Ident;

pub fn quote_struct(export: &NamedType, schema: &Struct, types: &TypeMap) -> TokenStream {
    if export.binding_style == BindingStyle::Handle {
        return class::quote_handle_type(export);
    }

    let ident = format_ident!("{}", &*export.name);
    let raw_ident = binding::raw_ident(&export.name);

    let fields = schema
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| (field_ident(Some(&field.0), index), &field.1))
        .collect::<Vec<_>>();

    let struct_fields = struct_fields(&fields, types);
    let raw_fields = binding::raw_struct_fields(&fields, types);

    quote! {
        public struct #ident
        {
            #struct_fields
        }

        internal struct #raw_ident
        {
            #raw_fields
        }
    }
}

/// Quotes the field declarations for the generated C# struct corresponding to an
/// exported Rust type.
pub fn struct_fields(fields: &[(Ident, &Schema)], types: &TypeMap) -> TokenStream {
    let field_ident = fields.iter().map(|(ident, _)| ident);
    let field_ty = fields
        .iter()
        .map(|(_, schema)| generate::quote_cs_type(schema, types));

    quote! {
        #(
            public #field_ty #field_ident;
        )*
    }
}

/// Converts the specified field name into a C#-appropriate ident, or generates an
/// ident based on the index of the field if the field is unnamed.
pub fn field_ident(name: Option<&str>, index: usize) -> Ident {
    name.map(|name| format_ident!("{}", name.to_camel_case()))
        .unwrap_or_else(|| format_ident!("Element{}", index))
}
