//! Code generation for exported functions and methods.

use super::quote_cs_type_for_repr;
use crate::generate::{binding, TypeMap};
use cs_bindgen_shared::*;
use heck::*;
use proc_macro2::TokenStream;
use quote::*;

pub fn quote_wrapper_fn<'a>(
    name: &str,
    binding: &str,
    receiver: Option<TokenStream>,
    inputs: &[FnArg],
    output: Option<&Repr>,
    types: &'a TypeMap,
) -> TokenStream {
    // Determine the name of the wrapper function. The original function name is
    // going to be in `snake_case`, so we need to convert it to `CamelCase` to keep
    // with C# naming conventions.
    let name = format_ident!("{}", name.to_camel_case());

    let return_ty = match output {
        Some(output) => quote_cs_type_for_repr(&output, types),
        None => quote! { void },
    };

    // Generate the declaration for the output variable and return expression. We need
    // to treat `void` returns as a special case, since C# won't let you declare values
    // with type `void` (*sigh*).
    let ret = quote! { __raw_result };
    let ret_decl = match output {
        Some(schema) => {
            let raw_return_ty = binding::raw_type_from_repr(schema, types);
            quote! { #raw_return_ty #ret; }
        }

        None => quote! {},
    };

    let binding_class = binding::bindings_class_ident();
    let from_raw = binding::from_raw_fn_ident();

    let ret_expr = match output {
        Some(_) => quote! {
            #binding_class.#from_raw(#ret, out #return_ty __result);
            return __result;
        },

        None => quote! {},
    };

    // Determine if the function should be static or not based on whether or not it has
    // a receiver.
    let static_ = if receiver.is_some() {
        TokenStream::default()
    } else {
        quote! { static }
    };

    let args = quote_args(inputs, types);
    let body = quote_wrapper_body(binding, receiver, &inputs, output.map(|_| &ret), types);

    quote! {
        public #static_ #return_ty #name(#( #args ),*)
        {
            unsafe {
                #ret_decl
                #body
                #ret_expr
            }
        }
    }
}

pub fn quote_wrapper_body<'a>(
    binding_name: &str,
    receiver: Option<TokenStream>,
    args: &[FnArg],
    output: Option<&TokenStream>,
    types: &TypeMap,
) -> TokenStream {
    let arg_name = args
        .iter()
        .map(|arg| format_ident!("{}", arg.name.to_mixed_case()));
    let temp_arg_name = args.iter().map(|arg| format_ident!("__{}", arg.name));
    let raw_ty = args
        .iter()
        .map(|arg| binding::raw_type_from_repr(&arg.repr, types));

    let bindings = binding::bindings_class_ident();
    let into_raw = binding::into_raw_fn_ident();

    // Build the list of arguments to the wrapper function and insert the receiver at
    // the beginning of the list of arguments if necessary.
    let mut invoke_arg = temp_arg_name
        .clone()
        .map(|name| name.into_token_stream())
        .collect::<Vec<_>>();
    if let Some(receiver) = receiver {
        invoke_arg.insert(0, receiver);
    }

    let raw_fn = format_ident!("{}", binding_name);

    // Generate the expression for invoking the raw function. If
    let invoke = quote! { #bindings.#raw_fn(#( #invoke_arg ),*) };

    let out_equals = match output {
        Some(output) => quote! { #output = },
        None => quote! {},
    };

    let body = quote! {
        #(
            #bindings.#into_raw(#arg_name, out #raw_ty #temp_arg_name);
        )*

        #out_equals #invoke;
    };

    fold_fixed_blocks(body, args)
}

fn fold_fixed_blocks<'a>(base_invoke: TokenStream, args: &[FnArg]) -> TokenStream {
    // Wrap the body of the function in `fixed` blocks for any parameters that need to
    // be passed as pointers to Rust (just strings for now). We use `Iterator::fold` to
    // generate a series of nested `fixed` blocks. This is very smart code and won't be
    // hard to maintain at all, I'm sure.
    args.iter().fold(base_invoke, |body, arg| {
        if arg.repr == Repr::String {
            let arg_ident = format_ident!("{}", arg.name.to_mixed_case());
            let fixed_ident = format_ident!("__fixed_{}", arg_ident);
            quote! {
                fixed (char* #fixed_ident = #arg_ident)
                {
                    #body
                }
            }
        } else {
            body
        }
    })
}

/// Generates the argument declarations for a C# wrapper function.
///
/// Attempts to use the most idiomatic C# type that corresponds to the original type.
pub fn quote_args<'a>(
    args: &'a [FnArg],
    types: &'a TypeMap<'_>,
) -> impl Iterator<Item = TokenStream> + 'a {
    args.iter().map(move |arg| {
        let ident = format_ident!("{}", arg.name.to_mixed_case());
        let ty = quote_cs_type_for_repr(&arg.repr, types);
        quote! { #ty #ident }
    })
}
