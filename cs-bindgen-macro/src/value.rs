use proc_macro2::TokenStream;
use quote::*;
use syn::*;

pub fn quote_abi_struct(ident: &Ident, fields: &Fields) -> TokenStream {
    // Extract the list of fields for the binding struct. The generated struct is the
    // same for both struct-like and tuple-like variants, though in the latter case we
    // have to manually generate names for the fields based on the index of the element.
    let from_fields = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let field_ty = &field.ty;
            let field_ident = field
                .ident
                .as_ref()
                .map(Clone::clone)
                .unwrap_or_else(|| format_ident!("element_{}", index));

            quote! {
                #field_ident: <#field_ty as cs_bindgen::abi::Abi>::Abi
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #[allow(bad_style)]
        pub struct #ident {
            #( #from_fields, )*
        }
    }
}
