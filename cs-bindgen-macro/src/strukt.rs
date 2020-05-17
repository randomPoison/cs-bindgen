use crate::{
    describe_named_type, handle, has_derive_copy, quote_index_fn, quote_vec_drop_fn,
    reject_generics, repr_impl, type_name_expr, value, BindingStyle,
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

    let repr_fn = repr_impl(&item.ident);

    // Determine whether we should marshal the type as a handle or by value.
    if has_derive_copy(&item.attrs)? {
        let describe_impl = describe_struct(&item);

        fn field_accessor(index: usize, field: &Field) -> TokenStream {
            field
                .ident
                .as_ref()
                .map(|ident| ident.into_token_stream())
                .unwrap_or_else(|| Literal::usize_unsuffixed(index).into_token_stream())
        }

        let abi_struct_ident = format_binding_ident!(item.ident);
        let abi_struct = value::quote_abi_struct(&abi_struct_ident, &item.fields);
        let describe_fn = describe_named_type(&item.ident, BindingStyle::Value);
        let index_fn = quote_index_fn(&item.ident);
        let vec_drop_fn = quote_vec_drop_fn(&item.ident);

        let into_abi_fields = value::into_abi_fields(&item.fields, |index, field| {
            let accessor = field_accessor(index, field);
            quote! { self.#accessor }
        });

        let as_abi_fields = value::as_abi_fields(&item.fields, |index, field| {
            let accessor = field_accessor(index, field);
            quote! { &self.#accessor }
        });

        // Generate the `from_abi` conversions for the fields, then wrap that code in the
        // appropriate kind of braces based on the style of the struct.
        let from_abi_fields = value::from_abi_fields(&item.fields, &quote! { abi });
        let from_abi_braces = match &item.fields {
            Fields::Named(_) => quote! { { #from_abi_fields } },
            Fields::Unnamed(_) => quote! { ( #from_abi_fields ) },
            Fields::Unit => quote! {},
        };

        let ident = item.ident;
        Ok(quote! {
            #abi_struct

            impl cs_bindgen::abi::Abi for #ident {
                type Abi = #abi_struct_ident;

                #repr_fn

                unsafe fn from_abi(abi: Self::Abi) -> Self {
                    Self #from_abi_braces
                }

                fn as_abi(&self) -> Self::Abi {
                    Self::Abi {
                        #as_abi_fields
                    }
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
            #vec_drop_fn
        })
    } else {
        handle::quote_type_as_handle(&item.ident)
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

    let type_name = type_name_expr(ident);

    quote! {
        impl cs_bindgen::shared::schematic::Describe for #ident {
            fn type_name() -> cs_bindgen::shared::TypeName {
                #type_name
            }

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
