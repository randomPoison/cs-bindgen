use crate::generate::{self, binding, quote_primitive_type, TypeMap};
use cs_bindgen_shared::{schematic::Enum, schematic::Variant, BindingStyle, NamedType};
use heck::*;
use proc_macro2::{Literal, TokenStream};
use quote::*;
use syn::Ident;

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
        let cs_repr = variant_struct_type_ref(export, variant);

        if variant.is_empty() {
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

pub fn into_raw_impl(export: &NamedType, schema: &Enum) -> TokenStream {
    // For C-like enums, the conversion is just invoking the constructor of the raw struct.
    if !schema.has_data() {
        let raw_ident = binding::raw_ident(&export.name);
        return quote! {
            return new #raw_ident(self);
        };
    }

    let raw_struct_ty = binding::raw_ident(&export.name);
    let union_ty = union_struct_name(&export.name);

    let variant_name = schema
        .variants
        .iter()
        .map(|variant| format_ident!("{}", &variant.name()));

    let variant_type = schema
        .variants
        .iter()
        .map(|variant| variant_struct_type_ref(export, variant));

    let discriminant = schema
        .variants
        .iter()
        .enumerate()
        .map(|(index, _)| Literal::usize_unsuffixed(index));

    let convert_union_field = schema.variants.iter().map(|variant| {
        // Empty variants aren't represented in the union, so leave the constructor body
        // empty.
        if variant.is_empty() {
            quote! {}
        } else {
            let variant_name = format_ident!("{}", variant.name());
            let raw_variant_type = raw_variant_struct_type_ref(export, variant);
            quote! {
                #variant_name = new #raw_variant_type(#variant_name)
            }
        }
    });

    quote! {
        switch (self)
        {
            #(
                case #variant_type #variant_name:
                {
                    return new #raw_struct_ty(
                        #discriminant,
                        new #union_ty() { #convert_union_field });
                }
            )*

            default:
                throw new Exception("Unrecognized enum variant: " + self);
        }
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

    let raw_ident = binding::raw_ident(&export.name);
    let discriminant_ty = quote_discriminant_type(schema);

    quote! {
        public enum #ident {
            #( #variants ),*
        }

        [StructLayout(LayoutKind.Explicit)]
        internal unsafe struct #raw_ident
        {
            [FieldOffset(0)]
            public #discriminant_ty Inner;

            public #raw_ident(#ident self)
            {
                this.Inner = (#discriminant_ty)self;
            }

            public static explicit operator #ident(#raw_ident raw)
            {
                return (#ident)raw.Inner;
            }
        }
    }
}

fn quote_complex_enum_binding(export: &NamedType, schema: &Enum, types: &TypeMap) -> TokenStream {
    assert_eq!(
        export.binding_style,
        BindingStyle::Value,
        "Right now we only support exporting complex enums by value"
    );

    let wrapper_class = format_ident!("{}", &*export.name);
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
                #wrapper_class.#binding_ty #name
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
        let ident = variant_struct_name(variant);
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
            let ty = generate::quote_cs_type(schema, types);
            quote! {
                #ty #field_ident
            }
        });

        let arg_binding_fields = fields.iter().map(|(field_ident, schema)| {
            let binding_ty = binding::quote_raw_type_reference(schema, types);

            quote! {
                #binding_ty #field_ident
            }
        });

        let constructor_fields = fields.iter().map(|(field_ident, _)| {
            let from_raw_fn = binding::from_raw_fn_ident();
            quote! {
                this.#field_ident = __bindings.#from_raw_fn(raw.#field_ident);
            }
        });

        let field_name = fields.iter().map(|(name, _)| name);

        quote! {
            // Generate the C# struct for the variant.
            public struct #ident : #interface
            {
                // Populate the fields of the variant struct.
                #(
                    public #struct_fields;
                )*

                // Generate an internal constructor for creating an instance of the variant struct
                // from its raw representation.
                internal #ident(#raw_ident raw)
                {
                    #(
                        #constructor_fields
                    )*
                }
            }

            // Generate the raw struct for the variant.
            [StructLayout(LayoutKind.Sequential)]
            internal struct #raw_ident
            {
                #(
                    internal #arg_binding_fields;
                )*

                // Generate a constructor that converts the C# representation of the variant into
                // its raw representation.
                public #raw_ident(#ident self)
                {
                    #(
                        this.#field_name = __bindings.__IntoRaw(self.#field_name);
                    )*
                }
            }
        }
    });

    let raw_struct = binding::raw_ident(&export.name);
    let union_struct = union_struct_name(&export.name);

    quote! {
        // Generate an interface for the enum.
        public interface #interface { }

        // Generate wrapper class in order to namespace the variants.
        public static class #wrapper_class
        {
            // Generate the struct declarations for each variant of the enum.
            #( #variant_structs )*
        }

        // Generate the raw struct, which contains the discriminant and a union of all the
        // possible variants. This needs to match the `RawEnum<D, V>` type on the Rust side.
        [StructLayout(LayoutKind.Sequential)]
        internal unsafe struct #raw_struct
        {
            public IntPtr Discriminant;
            public #union_struct Value;

            public #raw_struct(int discriminant, #union_struct value)
            {
                this.Discriminant = new IntPtr(discriminant);
                this.Value = value;
            }

            public #raw_struct(long discriminant, #union_struct value)
            {
                this.Discriminant = new IntPtr(discriminant);
                this.Value = value;
            }

            public #raw_struct(IntPtr discriminant, #union_struct value)
            {
                this.Discriminant = discriminant;
                this.Value = value;
            }
        }

        // Generate a struct that acts as a union of all the data-carrying variants.
        [StructLayout(LayoutKind.Explicit)]
        internal struct #union_struct
        {
            #(
                [FieldOffset(0)]
                internal #union_fields;
            )*
        }
    }
}

/// Returns the name of the wrapper class generated for for the specified exported type.
fn wrapper_class_name(export: &NamedType) -> Ident {
    format_ident!("{}", &*export.name)
}

/// Returns the name for the C# struct representing the specified variant.
///
/// Note that this only returns the name of the variant, not the fully-qualified
/// path to the type. If you need a fully-qualified type-reference (e.g. because
/// you're generating code that needs to interact with a value of the variant), use
/// [`variant_struct_type_ref`] instead.
///
/// [`variant_struct_type_ref`]: fn.variant_struct_type_ref.html
fn variant_struct_name(variant: &Variant) -> Ident {
    format_ident!("{}", variant.name())
}

/// Generates a type reference to the C# type for the specified enum variant.
fn variant_struct_type_ref(export: &NamedType, variant: &Variant) -> TokenStream {
    let wrapper_class = wrapper_class_name(export);
    let variant_struct_name = variant_struct_name(variant);
    quote! {
        global::#wrapper_class.#variant_struct_name
    }
}

pub fn raw_variant_struct_type_ref(export: &NamedType, variant: &Variant) -> TokenStream {
    let wrapper_class = wrapper_class_name(export);
    let raw_variant_struct_name = binding::raw_ident(variant.name());
    quote! {
        global::#wrapper_class.#raw_variant_struct_name
    }
}

fn union_struct_name(name: &str) -> Ident {
    format_ident!("{}_Data_Raw", name)
}
