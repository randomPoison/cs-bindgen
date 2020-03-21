//! Helper functions for generating raw bindings and descriptor functions.

use proc_macro2::TokenStream;
use quote::*;
use std::fmt::Display;
use syn::{punctuated::Punctuated, token::Comma, *};

type FnInput = (Ident, Box<Type>);

/// Generates an error if any generic parameters are present.
///
/// In general we can't support `#[cs_bindgen]` on generic items, any item that
/// supports generic parameters needs to generate an error during parsing. This
/// helper method can be used to check the `Generics` AST node that syn generates
/// and will return an error if the node contains any generic parameters.
pub fn reject_generics<M: Display>(generics: &Generics, message: M) -> syn::Result<()> {
    let has_generics = generics.type_params().next().is_some()
        || generics.lifetimes().next().is_some()
        || generics.const_params().next().is_some();
    if has_generics {
        Err(Error::new_spanned(generics, message))
    } else {
        Ok(())
    }
}

/// Processes the raw list of arguments into a format suitable for use in code
/// generation.
///
/// Extracts all of the non-receiver arguments (i.e. everything but the initial
/// `self` argument), converting them into an `(ident, type)` pair. Any argument
/// that doesn't already have an ident (i.e. because it uses a pattern instead an
/// ident) will have one generated in order to ensure that all arguments have a
/// valid ident.
pub fn extract_inputs(inputs: Punctuated<FnArg, Comma>) -> syn::Result<Vec<FnInput>> {
    inputs
        .into_iter()
        // Convert the `FnArg` arguments into the underlying `PatType`. This is safe to do
        // in this context because we know we are processing a free function, so it cannot
        // have a receiver.
        .filter_map(|arg| match arg {
            FnArg::Typed(arg) => Some(arg),
            _ => None,
        })
        .enumerate()
        .map(|(index, arg)| {
            // If the argument isn't declared with a normal identifier, we construct one so
            // that we have a valid identifier to use in the generated functions.
            let ident = match *arg.pat {
                Pat::Ident(pat_ident) => pat_ident.ident,
                _ => format_ident!("__arg{}", index),
            };

            Ok((ident, arg.ty))
        })
        .collect()
}

/// Generates the declaration for an argument to the binding function.
///
/// This function takes the ident and type of an argument in the original function
/// and generates the `ident: type` declaration for the corresponding argument in
/// the binding function. The ident is reused directly, and `Abi` associated type
/// on the `Abi` impl for `ty` is used as the type of the generated argument.
pub fn quote_binding_inputs<T: ToTokens>(ident: &Ident, ty: T) -> TokenStream {
    quote! {
        #ident: <#ty as cs_bindgen::abi::Abi>::Abi
    }
}

/// Generates the call to `Abi::from_abi` to convert the raw binding argument.
pub fn quote_input_conversion(ident: &Ident) -> TokenStream {
    quote! {
        let #ident = cs_bindgen::abi::Abi::from_abi(#ident);
    }
}

// /// Extracts the specified return type for the function, explicitly using `()` for
// /// the default return.
// ///
// /// This simplifies the logic needed for quoting the return type of the binding
// /// function, since it removes the need to distinguish between an explicit return vs
// /// the default return (i.e. returning `()`).
// pub fn normalize_return_type(output: &ReturnType) -> TokenStream {
//     // TODO: Generate an error for `impl trait` returns.
//     match output {
//         ReturnType::Default => quote! { () },
//         ReturnType::Type(_, ty) => ty.to_token_stream(),
//     }
// }
