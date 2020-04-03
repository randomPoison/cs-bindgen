use proc_macro2::{Literal, TokenStream};
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
                .clone()
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

        unsafe impl cs_bindgen::abi::AbiPrimitive for #ident {}
    }
}

pub fn into_abi_fields(fields: &Fields, input: &TokenStream) -> TokenStream {
    let abi_field = fields.iter().enumerate().map(|(index, field)| {
        field
            .ident
            .clone()
            .unwrap_or_else(|| format_ident!("element_{}", index))
    });

    let conversion = fields.iter().enumerate().map(|(index, field)| {
        let input_field = field
            .ident
            .as_ref()
            .map(|ident| ident.to_token_stream())
            .unwrap_or_else(|| Literal::usize_unsuffixed(index).into_token_stream());
        quote! {
            cs_bindgen::abi::Abi::into_abi(#input.#input_field)
        }
    });

    quote! {
        #(
            #abi_field: #conversion,
        )*
    }
}

pub fn from_abi_fields(fields: &Fields, input: &TokenStream) -> TokenStream {
    let assignment = fields.iter().map(|field| match &field.ident {
        Some(ident) => quote! { #ident: },
        None => quote! {},
    });

    let conversion = fields.iter().enumerate().map(|(index, field)| {
        let field_ident = field
            .ident
            .clone()
            .unwrap_or_else(|| format_ident!("element_{}", index));

        quote! { cs_bindgen::abi::Abi::from_abi(#input.#field_ident) }
    });

    quote! {
        #(
            #assignment #conversion,
        )*
    }
}
