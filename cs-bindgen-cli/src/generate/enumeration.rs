use cs_bindgen_shared::{schematic, schematic::Variant, Enum};
use proc_macro2::TokenStream;
use quote::*;

pub fn quote_enum_binding(item: &Enum) -> TokenStream {
    let schema = item
        .schema
        .as_enum()
        .expect("Enum item's schema does not describe an enum");

    // Determine if we're dealing with a simple (C-like) enum or one with fields.
    if schema.has_data() {
        // TODO: Generate bindings for enums with data.
        quote! {}
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
