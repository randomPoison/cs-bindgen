use cs_bindgen_shared::*;
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma, Ident};

pub fn quote_bindgen_fn(bindgen_fn: &BindgenFn, dll_name: &str) -> TokenStream {
    let wrapper_fn = quote_wrapper_fn(&bindgen_fn);
    let raw_binding = quote_raw_binding(&bindgen_fn, dll_name);

    quote! {
        #raw_binding
        #wrapper_fn
    }
}

pub fn quote_raw_binding(bindgen_fn: &BindgenFn, dll_name: &str) -> TokenStream {
    let entry_point = bindgen_fn.generated_name();
    let binding_return_ty = match bindgen_fn.ret.primitive() {
        None => quote! { void },
        Some(prim) => match prim {
            Primitive::String => quote! { RustOwnedString },
            Primitive::Char => quote! { uint },
            Primitive::I8 => quote! { sbyte },
            Primitive::I16 => quote! { short },
            Primitive::I32 => quote! { int },
            Primitive::I64 => quote! { long },
            Primitive::U8 => quote! { byte },
            Primitive::U16 => quote! { ushort },
            Primitive::U32 => quote! { uint },
            Primitive::U64 => quote! { ulong },
            Primitive::F32 => quote! { float },
            Primitive::F64 => quote! { double },
            Primitive::Bool => quote! { byte },
        },
    };

    let mut binding_args = bindgen_fn
        .args
        .iter()
        .map(|arg| {
            let ident = arg.ident();
            let ty = quote_primitive_binding_arg(arg.ty);
            quote! { #ty #ident }
        })
        .collect::<Punctuated<_, Comma>>();

    if bindgen_fn.receiver.is_some() {
        binding_args.insert(0, quote! { *void self })
    }

    let raw_ident = bindgen_fn.generated_ident();
    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
        private static extern #binding_return_ty #raw_ident(#binding_args);
    }
}

pub fn quote_wrapper_body(bindgen_fn: &BindgenFn, output_ident: &Ident) -> TokenStream {
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

    let raw_ident = bindgen_fn.generated_ident();
    let invoke_expr = match bindgen_fn.ret.primitive() {
        None => quote! { #raw_ident(#invoke_args); },

        Some(prim) => {
            let invoke_expr = quote! { var rawResult = #raw_ident(#invoke_args); };

            let result_expr = match prim {
                Primitive::String => quote! {
                    string result = Encoding.UTF8.GetString(rawResult.Ptr, (int)rawResult.Length);
                    DropString(rawResult);
                    #output_ident = result;
                },

                Primitive::Bool => quote! {
                    #output_ident = rawResult != 0;
                },

                _ => quote! { #output_ident = rawResult; },
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

pub fn quote_wrapper_args(bindgen_fn: &BindgenFn) -> Punctuated<TokenStream, Comma> {
    let mut args = bindgen_fn
        .args
        .iter()
        .map(|arg| {
            let ident = arg.ident();
            let ty = quote_primitive(arg.ty);
            quote! { #ty #ident }
        })
        .collect::<Punctuated<_, Comma>>();

    // Insert additional self parameter if the function has a receiver.
    if bindgen_fn.receiver.is_some() {
        args.insert(0, quote! { void* self });
    }

    args
}

pub fn quote_wrapper_fn(bindgen_fn: &BindgenFn) -> TokenStream {
    let cs_fn_name = format_ident!("{}", bindgen_fn.raw_ident().to_camel_case());
    let cs_return_ty = match bindgen_fn.ret.primitive() {
        None => quote! { void },
        Some(prim) => quote_primitive(prim),
    };

    let ret = format_ident!("__ret");

    let args = quote_wrapper_args(bindgen_fn);
    let body = quote_wrapper_body(bindgen_fn, &ret);

    let static_ = if bindgen_fn.receiver.is_none() {
        quote! { static }
    } else {
        TokenStream::default()
    };

    quote! {
        public #static_ #cs_return_ty #cs_fn_name(#args)
        {
            #cs_return_ty #ret;
            unsafe {
                // TODO: Process args so they're ready to pass to the rust fn.

                #body
            }
            return #ret;
        }
    }
}

/// Quotes the C# type for an argument to the raw binding function.
pub fn quote_primitive_binding_arg(arg_ty: Primitive) -> TokenStream {
    match arg_ty {
        Primitive::String => quote! { RawCsString },
        Primitive::Char => quote! { uint },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::F32 => quote! { float },
        Primitive::F64 => quote! { double },
        Primitive::Bool => quote! { byte },
    }
}

/// Quotes the idiomatic C# type corresponding to a given primitive type.
pub fn quote_primitive(return_ty: Primitive) -> TokenStream {
    match return_ty {
        Primitive::String => quote! { string },
        Primitive::Char => quote! { uint },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::F32 => quote! { float },
        Primitive::F64 => quote! { double },
        Primitive::Bool => quote! { bool },
    }
}
