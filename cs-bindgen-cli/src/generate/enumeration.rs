use crate::generate::{binding, class, quote_cs_type, quote_primitive_type, TypeMap};
use cs_bindgen_shared::{schematic::Enum, schematic::Variant, BindingStyle, NamedType};
use heck::*;
use proc_macro2::{Literal, TokenStream};
use quote::*;

pub fn quote_enum_binding(export: &NamedType, schema: &Enum, types: &TypeMap) -> TokenStream {
    // Determine if we're dealing with a simple (C-like) enum or one with fields.
    if schema.has_data() {
        quote_complex_enum_binding(export, schema, types)
    } else {
        quote_simple_enum_binding(export, schema)
    }
}

pub fn quote_type_reference(export: &NamedType, schema: &Enum) -> TokenStream {
    if export.binding_style == BindingStyle::Value && schema.has_data() {
        format_ident!("I{}", &*export.name).into_token_stream()
    } else {
        format_ident!("{}", &*export.name).into_token_stream()
    }
}

/// Quotes the name of the generated C# type for the exported enum.
///
/// There are three possible raw representations for an enum:
///
/// * For C-like enums, the raw representation is the integer type used for the
///   enum's discriminant.
/// * For data-carrying enums that are marshaled by value, the raw representation is
/// * For enums that are marshalled as handles, the raw representation is just the
///   handle pointer type (`void*`).
pub fn quote_raw_type_reference(export: &NamedType, schema: &Enum) -> TokenStream {
    match export.binding_style {
        BindingStyle::Value => {
            if schema.has_data() {
                let union_ty = binding::raw_ident(&export.name);
                quote! {
                    RawEnum<#union_ty>
                }
            } else {
                quote_discriminant_type(schema)
            }
        }

        BindingStyle::Handle => class::quote_handle_ptr(),
    }
}

/// Quotes the appropriate discriminant type for the specified enum type.
///
/// The generated type is the type use to represent the raw discriminant when
/// communicating with Rust. On the C# side, C-like enums are always represented as
/// `int` under the hood, and complex enums don't have a specific discriminant since
/// they are represented using an interface.
pub fn quote_discriminant_type(schema: &Enum) -> TokenStream {
    schema
        .repr
        .map(quote_primitive_type)
        .unwrap_or_else(|| quote! { IntPtr })
}

pub fn from_raw_impl(export: &NamedType, schema: &Enum) -> TokenStream {
    // For C-like enums, the conversion is just casting the raw discriminant value to
    // the C# enum type.
    if !schema.has_data() {
        let cs_repr = quote_type_reference(export, schema);
        return quote! { return (#cs_repr)raw; };
    }

    let discriminants = schema
        .variants
        .iter()
        .enumerate()
        .map(|(index, _)| Literal::usize_unsuffixed(index));

    let convert_variants = schema.variants.iter().map(|variant| {
        let cs_repr = raw_variant_struct_name(variant.name());

        if variant.is_empty() {
            println!("Variant {:?} is empty", variant);
            quote! {
                return new #cs_repr();
            }
        } else {
            let union_field = format_ident!("{}", variant.name());
            quote! {
                return new #cs_repr(raw.Value.#union_field);
            }
        }
    });

    quote! {
        switch (raw.Discriminant.ToInt64())
        {
            #(
                case #discriminants:
                {
                    #convert_variants
                }
            )*

            default: throw new Exception("Invalid discriminant " + raw.Discriminant);
        }
    }
}

pub fn into_raw_impl(_export: &NamedType, _schema: &Enum) -> TokenStream {
    quote! {
        throw new NotImplementedException("Convert enum to raw representation");
    }
}

fn quote_simple_enum_binding(export: &NamedType, schema: &Enum) -> TokenStream {
    let ident = format_ident!("{}", &*export.name);
    let variants = schema.variants.iter().map(|variant| {
        let (name, discriminant) = match variant {
            Variant::Unit { name, discriminant } => (name, discriminant),

            _ => panic!("Simple enum can only have unit variants"),
        };

        let variant_ident = format_ident!("{}", &**name);
        let discriminant = match discriminant {
            Some(discriminant) => {
                let lit = syn::parse_str::<syn::Expr>(&discriminant.to_string())
                    .expect("Failed to parse discriminant as a `LitInt`");
                quote! { = #lit }
            }
            None => TokenStream::new(),
        };

        quote! {
            #variant_ident #discriminant
        }
    });

    quote! {
        public enum #ident {
            #( #variants ),*
        }
    }
}

fn quote_complex_enum_binding(export: &NamedType, schema: &Enum, types: &TypeMap) -> TokenStream {
    assert_eq!(
        export.binding_style,
        BindingStyle::Value,
        "Right now we only support exporting complex enums by value"
    );

    let interface = format_ident!("I{}", &*export.name);

    // Generate the declarations for the fields of the raw union. There's one field for
    // each data-carrying variant of the enum, i.e. unit-like variants don't have a
    // corresponding field in the union.
    let union_fields = schema.variants.iter().filter_map(|variant| match variant {
        Variant::Unit { .. } => None,
        _ => {
            let binding_ty = binding::raw_ident(variant.name());
            let name = format_ident!("{}", variant.name());

            Some(quote! {
                #binding_ty #name
            })
        }
    });

    // Generate the struct declarations for each variant of the enum. We generate two
    // structs for each variant:
    //
    // * The public struct that acts as the C# representation of the variant.
    // * The raw representation which is kept internal and used as a field of the raw
    //   union for the enum.
    let variant_structs = schema.variants.iter().map(|variant| {
        let ident = raw_variant_struct_name(variant.name());
        let raw_ident = binding::raw_ident(variant.name());

        let fields = variant
            .fields()
            .enumerate()
            .map(|(index, field)| {
                let field_ident = field
                    .name
                    .as_ref()
                    .map(|name| format_ident!("{}", name.to_camel_case()))
                    .unwrap_or_else(|| format_ident!("Element{}", index));

                (field_ident, field.schema)
            })
            .collect::<Vec<_>>();

        let struct_fields = fields.iter().map(|(field_ident, schema)| {
            let ty = quote_cs_type(schema, types);
            quote! {
                public #ty #field_ident
            }
        });

        let arg_binding_fields = fields.iter().map(|(field_ident, schema)| {
            let binding_ty = binding::quote_type_binding(schema, types);

            quote! {
                internal #binding_ty #field_ident
            }
        });

        quote! {
            // Generate the C# struct for the variant.
            public struct #ident : #interface
            {
                #( #struct_fields; )*
            }

            // Generate the raw struct for the variant.
            [StructLayout(LayoutKind.Sequential)]
            internal struct #raw_ident
            {
                #( #arg_binding_fields; )*
            }
        }
    });

    let raw_union = binding::raw_ident(&export.name);

    quote! {
        // Generate an interface for the enum.
        public interface #interface {}

        // Generate the struct declarations for each variant of the enum.
        #( #variant_structs )*

        // Generate the binding "unions" for args/returns.
        [StructLayout(LayoutKind.Explicit)]
        internal struct #raw_union
        {
            #(
                [FieldOffset(0)]
                internal #union_fields;
            )*
        }
    }
}

fn raw_variant_struct_name(name: &str) -> TokenStream {
    format_ident!("{}", name).into_token_stream()
}
