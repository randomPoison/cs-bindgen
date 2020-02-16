use cs_bindgen_shared::*;
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma, Ident};

pub fn quote_wrapper_body(bindgen_fn: &Func, binding_ident: &Ident, output: &Ident) -> TokenStream {
    // Build the list of arguments to the wrapper function.
    let mut invoke_args = bindgen_fn
        .args
        .iter()
        .map(|arg| match arg.ty {
            // To pass a string to Rust, we convert it into a `RawCsString` with the fixed pointer.
            // The code for wrapping the body of the function in a `fixed` block is done below,
            // since we need to generate the contents of the block first.
            Primitive::String => {
                let arg_ident = arg.ident();
                let fixed_ident = format_ident!("__fixed_{}", arg.raw_ident()).into_token_stream();
                quote! {
                    new RawCsString() { Ptr = #fixed_ident, Length = #arg_ident.Length, }
                }
            }

            Primitive::Bool => {
                let ident = arg.ident();
                quote! { (#ident ? 1 : 0) }
            }

            _ => arg.ident().into_token_stream(),
        })
        .collect::<Punctuated<_, Comma>>();

    if bindgen_fn.receiver.is_some() {
        invoke_args.insert(0, quote! { _handle });
    }

    let raw_binding = {
        let raw_ident = binding_ident;
        quote! { __bindings.#raw_ident }
    };

    let invoke_expr = match bindgen_fn.ret {
        ReturnType::Default => quote! { #raw_binding(#invoke_args); },

        ReturnType::SelfType => quote! {
            #output = #raw_binding(#invoke_args);
        },

        ReturnType::Primitive(prim) => {
            let invoke_expr = quote! { var rawResult = #raw_binding(#invoke_args); };

            let result_expr = match prim {
                Primitive::String => quote! {
                    string result = Encoding.UTF8.GetString(rawResult.Ptr, (int)rawResult.Length);
                    __bindings.__cs_bindgen_drop_string(rawResult);
                    #output = result;
                },

                Primitive::Bool => quote! {
                    #output = rawResult != 0;
                },

                _ => quote! { #output = rawResult; },
            };

            quote! {
                #invoke_expr

                #result_expr
            }
        }
    };

    // Wrap the body of the function in `fixed` blocks for any parameters that need to
    // be passed as pointers to Rust (just strings for now). We use `Iterator::fold` to
    // generate a series of nested `fixed` blocks. This is very smart code and won't be
    // hard to maintain at all, I'm sure.
    bindgen_fn
        .args
        .iter()
        .fold(invoke_expr, |body, arg| match arg.ty {
            Primitive::String => {
                let arg_ident = arg.ident();
                let fixed_ident = format_ident!("__fixed_{}", arg.raw_ident()).into_token_stream();
                quote! {
                    fixed (char* #fixed_ident = #arg_ident)
                    {
                        #body
                    }
                }
            }

            _ => body,
        })
}

pub fn quote_wrapper_args(bindgen_fn: &Func) -> Punctuated<TokenStream, Comma> {
    bindgen_fn
        .args
        .iter()
        .map(|arg| {
            let ident = arg.ident();
            let ty = quote_primitive(arg.ty);
            quote! { #ty #ident }
        })
        .collect()
}

pub fn quote_wrapper_fn(bindgen_fn: &Func, binding_ident: &Ident) -> TokenStream {
    let cs_fn_name = format_ident!("{}", bindgen_fn.raw_ident().to_camel_case());
    let cs_return_ty = match bindgen_fn.ret.primitive() {
        None => quote! { void },
        Some(prim) => quote_primitive(prim),
    };

    // Generate the declaration for the output variable and return expression. We need
    // to treat `void` returns as a special case, since C# won't let you declare values
    // with type `void` (*sigh*).
    let ret = format_ident!("__ret");
    let ret_decl = match bindgen_fn.ret {
        ReturnType::Default => TokenStream::default(),
        _ => quote! { #cs_return_ty #ret; },
    };
    let ret_expr = match bindgen_fn.ret {
        ReturnType::Default => TokenStream::default(),
        _ => quote! { return #ret; },
    };

    let args = quote_wrapper_args(bindgen_fn);
    let body = quote_wrapper_body(bindgen_fn, binding_ident, &ret);

    let static_ = if bindgen_fn.receiver.is_none() {
        quote! { static }
    } else {
        TokenStream::default()
    };

    quote! {
        public #static_ #cs_return_ty #cs_fn_name(#args)
        {
            #ret_decl
            unsafe {
                // TODO: Process args so they're ready to pass to the rust fn.

                #body
            }
            #ret_expr
        }
    }
}
