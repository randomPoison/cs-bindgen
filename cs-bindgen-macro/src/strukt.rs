use crate::{
    describe_named_type, handle, has_derive_copy, quote_index_fn, reject_generics, value,
    BindingStyle,
};
use proc_macro2::{Literal, TokenStream};
use quote::*;
use syn::*;

/// Generates the bindings for an exported struct.
pub fn quote_struct_item(item: ItemStruct) -> syn::Result<TokenStream> {
    reject_generics(
        &item.generics,
        "Generic structs are not supported with `#[cs_bindgen]`",
    )?;

    let describe_impl = describe_struct(&item);

    // Determine whether we should marshal the type as a handle or by value.
    if has_derive_copy(&item.attrs)? {
        let abi_struct_ident = format_binding_ident!(item.ident);
        let abi_struct = value::quote_abi_struct(&abi_struct_ident, &item.fields);
        let into_abi_fields = value::into_abi_fields(&item.fields, |index, field| {
            let accessor = field
                .ident
                .as_ref()
                .map(|ident| ident.into_token_stream())
                .unwrap_or_else(|| Literal::usize_unsuffixed(index).into_token_stream());
            quote! { self.#accessor }
        });
        let describe_fn = describe_named_type(&item.ident, BindingStyle::Value);
        let ident = item.ident;

        // Generate the `from_abi` conversions for the fields, then wrap that code in the
        // appropriate kind of braces based on the style of the struct.
        let from_abi_fields = value::from_abi_fields(&item.fields, &quote! { abi });
        let from_abi_braces = match &item.fields {
            Fields::Named(_) => quote! { { #from_abi_fields } },
            Fields::Unnamed(_) => quote! { ( #from_abi_fields ) },
            Fields::Unit => quote! {},
        };

        let index_fn = quote_index_fn(&ident)?;

        Ok(quote! {
            #abi_struct

            impl cs_bindgen::abi::Abi for #ident {
                type Abi = #abi_struct_ident;

                unsafe fn from_abi(abi: Self::Abi) -> Self {
                    Self #from_abi_braces
                }

                fn into_abi(self) -> Self::Abi {
                    Self::Abi {
                        #into_abi_fields
                    }
                }
            }

            #describe_impl
            #describe_fn
            #index_fn
        })
    } else {
        let binding = handle::quote_type_as_handle(&item.ident)?;
        Ok(quote! {
            #binding
            #describe_impl
        })
    }
}

fn describe_struct(item: &ItemStruct) -> TokenStream {
    let ident = &item.ident;

    let body = if item.fields.is_empty() {
        quote! {
            cs_bindgen::shared::schematic::Describer::describe_unit_struct(type_name)
        }
    } else {
        match &item.fields {
            // For tuple-like structs we have two cases to consider:
            //
            // * If the struct only has one element, then it's considered a newtype struct in
            //   the schematic data model.
            // * For any other number of elements, it is considered a tuple struct.
            //
            // An empty tuple-like struct is treated like a unit-struct in the data model,
            // though that case is already handled above when we check `item.fields.is_empty()`.
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() == 1 {
                    let inner = &fields.unnamed[0].ty;
                    quote! {
                        describer.describe_newtype_struct::<#inner>(type_name)
                    }
                } else {
                    let element_ty = fields.unnamed.iter().map(|field| &field.ty);
                    quote! {
                        let mut describer = describer.describe_tuple_struct(type_name)?;
                        #(
                            describer.describe_element::<#element_ty>()?;
                        )*
                        describer.end()
                    }
                }
            }

            // Normal structs (i.e. with named fields) are always considered structs in the data
            // model. The only exception being one with no fields, though that case is already
            // handled above when we check `item.fields.is_empty()`.
            Fields::Named(fields) => {
                let field_name = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap().to_string());
                let field_ty = fields.named.iter().map(|field| &field.ty);
                quote! {
                    let mut describer = describer.describe_struct(type_name)?;
                    #(
                        describer.describe_field::<#field_ty>(#field_name)?;
                    )*
                    describer.end()
                }
            }

            Fields::Unit => unreachable!("Empty struct bodies have already been handled"),
        }
    };

    quote! {
        impl cs_bindgen::shared::schematic::Describe for #ident {
            fn describe<D>(describer: D) -> Result<D::Ok, D::Error>
            where
                D: cs_bindgen::shared::schematic::Describer,
            {
                use cs_bindgen::shared::schematic::{Describer, DescribeStruct, DescribeTupleStruct};

                let type_name = cs_bindgen::shared::schematic::type_name!(#ident);
                #body
            }
        }
    }
}
