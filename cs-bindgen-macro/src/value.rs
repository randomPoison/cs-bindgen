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
            let field_ident = raw_field_ident(index, field);

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

/// Generates field conversion logic for an `Abi::into_abi` implementation.
///
/// Generates a comma-separated list of expressions to convert the specified fields
/// into their FFI-compatible representations. For named fields, this will look like:
///
/// ```ignore
/// field_name: cs_bindgen::abi::Abi::into_abi(prefix.field_name),
/// ```
///
/// Unnamed fields will be the same but without the explicit field name:
///
/// ```ignore
/// cs_bindgen::abi::Abi::into_abi(prefix.field_name),
/// ```
///
/// Returns an empty token stream if `fields` is empty.
///
/// # Field Accessors
///
/// `field_accessor` is a closure that should generate the appropriate expression
/// for accessing the field in the current context. The returned value will be
/// treated as the expression that is passed into `Abi::into_abi` in the generated
/// code.
pub fn into_abi_fields(
    fields: &Fields,
    field_accessor: impl Fn(usize, &Field) -> TokenStream,
) -> TokenStream {
    let abi_field = fields
        .iter()
        .enumerate()
        .map(|(index, field)| raw_field_ident(index, field));

    let conversion = fields.iter().enumerate().map(|(index, field)| {
        let input_field = field_accessor(index, field);
        quote! {
            cs_bindgen::abi::Abi::into_abi(#input_field)
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
        let field_ident = raw_field_ident(index, field);
        quote! { cs_bindgen::abi::Abi::from_abi(#input.#field_ident) }
    });

    quote! {
        #(
            #assignment #conversion,
        )*
    }
}

/// Returns the ident of the field in a raw binding struct corresponding to the
/// specified field
///
/// Automatically generates a valid identifier if the field does not already have
/// one. This ensures a consistent naming convention when generating struct
/// representations of enums. Unnamed fields will be named `element_{index}`.
pub fn raw_field_ident(index: usize, field: &Field) -> Ident {
    field
        .ident
        .as_ref()
        .map(Clone::clone)
        .unwrap_or_else(|| format_ident!("element_{}", index))
}
