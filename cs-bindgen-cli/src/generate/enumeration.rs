use crate::generate::quote_cs_type;
use cs_bindgen_shared::{schematic, schematic::Variant, BindingStyle, Enum};
use proc_macro2::TokenStream;
use quote::*;

pub fn quote_enum_binding(item: &Enum) -> TokenStream {
    let schema = item
        .schema
        .as_enum()
        .expect("Enum item's schema does not describe an enum");

    // Determine if we're dealing with a simple (C-like) enum or one with fields.
    if schema.has_data() {
        quote_complex_enum_binding(item, schema)
    } else {
        quote_simple_enum_binding(item, schema)
    }
}

fn quote_simple_enum_binding(item: &Enum, schema: &schematic::Enum) -> TokenStream {
    let ident = format_ident!("{}", &*item.name);
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

fn quote_complex_enum_binding(item: &Enum, schema: &schematic::Enum) -> TokenStream {
    assert_eq!(
        item.binding_style,
        BindingStyle::Value,
        "Right now we only support exporting complex enums by value"
    );

    let interface = format_ident!("I{}", &*item.name);

    let variant_structs = schema.variants.iter().map(|variant| {
        let ident = format_ident!("{}", &*variant.name());

        let fields = variant.fields().enumerate().map(|(index, field)| {
            let field_ident = field
                .name
                .as_ref()
                .map(|name| format_ident!("{}", name))
                .unwrap_or_else(|| format_ident!("element_{}", index));

            let ty = quote_cs_type(field.schema);

            quote! {
                public #ty #field_ident
            }
        });

        quote! {
            public struct #ident : #interface {
                #( #fields; )*
            }
        }
    });

    quote! {
        // Generate an interface for the enum.
        public interface #interface {}

        // Generate the struct declarations for each variant of the enum.
        #( #variant_structs )*
    }
}
