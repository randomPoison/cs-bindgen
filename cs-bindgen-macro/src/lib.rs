extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::*;
use serde::*;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    *,
};

#[proc_macro_attribute]
pub fn cs_bindgen(
    _attr: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Create a copy of the input token stream that we can later extend with the
    // generated code. This allows us to consume the input stream without needing to
    // manually reconstruct the original input later when returning the result.
    let orig: TokenStream = tokens.clone().into();

    let input = parse_macro_input!(tokens as BindgenFn);

    let result = quote! {
        #orig

        #input
    };

    result.into()
}

/// The return type of a function marked with `#[cs_bindgen]`.
///
/// This enum is similar to the syn `ReturnType` enum, but provides an additional
/// `Primitive` variant. This allows us to specifically identify primitive types
/// that can be passed across the FFI boundary without additional marshalling (or at
/// least without the complexity of fully describing the type).
#[derive(Debug, Clone)]
enum ReturnType {
    Default,
    Primitive(Primitive),
}

impl ReturnType {
    fn into_primitive(self) -> Option<Primitive> {
        match self {
            ReturnType::Default => None,
            ReturnType::Primitive(prim) => Some(prim),
        }
    }
}

impl Parse for ReturnType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ret: syn::ReturnType = input.parse()?;
        let inner = match ret {
            syn::ReturnType::Default => return Ok(ReturnType::Default),
            syn::ReturnType::Type(_, inner) => inner,
        };

        match Primitive::from_type(&inner) {
            Some(prim) => Ok(ReturnType::Primitive(prim)),
            None => Err(syn::Error::new(
                inner.span(),
                "Unsupported return type, only primitive types and `String` are supported",
            )),
        }
    }
}

/// A "known" Rust type that can be directly marshalled across the FFI boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Primitive {
    String,
    Char,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
}

impl Primitive {
    fn from_type(ty: &Type) -> Option<Self> {
        let ident = match &*ty {
            Type::Path(path) => match path.path.get_ident() {
                Some(ident) => ident,
                None => return None,
            },

            _ => return None,
        };

        let prim = match &*ident.to_string() {
            "String" => Primitive::String,
            "char" => Primitive::Char,
            "i8" => Primitive::I8,
            "i16" => Primitive::I16,
            "i32" => Primitive::I32,
            "i64" => Primitive::I64,
            "u8" => Primitive::U8,
            "u16" => Primitive::U16,
            "u32" => Primitive::U32,
            "u64" => Primitive::U64,
            "f32" => Primitive::F32,
            "f64" => Primitive::F64,
            "bool" => Primitive::Bool,

            _ => return None,
        };

        Some(prim)
    }

    /// Generates the code for returning the final result of the function.
    fn generate_return_expr(&self, ret_val: &Ident, args: &mut Vec<TokenStream>) -> TokenStream {
        match self {
            // S
            Primitive::String => {
                // Generate the out param for the length of the string.
                let out_param = format_ident!("out_len");
                args.push(quote! {
                    #out_param: *mut i32
                });

                // Generate the code for
                quote! {
                    *#out_param = #ret_val
                        .len()
                        .try_into()
                        .expect("String length is too large for `i32`");

                    std::ffi::CString::new(#ret_val)
                        .expect("Generated string contained a null byte")
                        .into_raw()
                }
            }

            // Cast the bool to a `u8` in order to pass it to C# as a numeric value.
            Primitive::Bool => quote! {
                #ret_val as u8
            },

            // All other primitive types are ABI-compatible with a corresponding C# type, and
            // require no extra processing to be returned.
            _ => quote! { #ret_val },
        }
    }
}

impl ToTokens for Primitive {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let result = match self {
            Primitive::String => quote! { *mut std::os::raw::c_char },
            Primitive::Char => quote! { u32 },
            Primitive::I8 => quote! { i8 },
            Primitive::I16 => quote! { i16 },
            Primitive::I32 => quote! { i32 },
            Primitive::I64 => quote! { i64 },
            Primitive::U8 => quote! { u8 },
            Primitive::U16 => quote! { u16 },
            Primitive::U32 => quote! { u32 },
            Primitive::U64 => quote! { u64 },
            Primitive::F32 => quote! { f32 },
            Primitive::F64 => quote! { f64 },
            Primitive::Bool => quote! { u8 },
        };

        tokens.append_all(result);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BindgenFnArg {
    ident: String,
    ty: Primitive,
}

#[derive(Debug, Serialize, Deserialize)]
struct BindgenFn {
    ident: String,
    args: Vec<Primitive>,
    ret: Option<Primitive>,
}

impl Parse for BindgenFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse attributes on the function.
        let _ = input.call(Attribute::parse_outer)?;

        // Parse the visibility specifier. We discard the result because we don't care about
        // visibility for now: The generated function always has to be public, so the visibility of
        // the original function doesn't matter.
        let _ = input.parse::<Visibility>();

        // Generate an error if the function is async.
        if let Ok(token) = input.parse::<Token![async]>() {
            return Err(syn::Error::new(
                token.span,
                "Async functions cannot be called by C# code",
            ));
        }

        input.parse::<Token![fn]>()?;
        let ident = input.parse::<Ident>()?.to_string();

        let content;
        parenthesized!(content in input);
        let args = content
            .parse_terminated::<FnArg, Comma>(FnArg::parse)?
            .iter()
            .map(|arg| match arg {
                // Reject any functions that take some form of `self`. We'll eventually be able to
                // support these by marking entire `impl` blocks with `#[cs_bindgen]`, but for now
                // we only support free functions.
                FnArg::Receiver(_) => Err(syn::Error::new(
                    arg.span(),
                    "Methods are not supported, only free functions",
                )),

                // Parse out just the type of the parameter. We'll want to preserve the name of the
                // param eventually in order to provide better naming in the generated C# code, but
                // that would require that we handle the case where the function param uses a
                // pattern rather than a regular identifier, and I don't feel like writing that code
                // right now.
                FnArg::Typed(pat) => Primitive::from_type(&pat.ty).ok_or(syn::Error::new(
                    arg.span(),
                    "Methods are not supported, only free functions",
                )),
            })
            .collect::<syn::Result<_>>()?;

        let ret = input.parse::<ReturnType>()?.into_primitive();

        // TODO: I guess this will probably break on `where` clauses?

        // NOTE: We must fully parse the body of the method in order to
        let content;
        braced!(content in input);
        let _ = content.call(Block::parse_within)?;

        Ok(BindgenFn { ident, args, ret })
    }
}

impl ToTokens for BindgenFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // Determine the name of the generated function.
        let generated_fn_ident = format_ident!("__cs_bindgen_generated_{}", self.ident);

        // Process the arguments to the function. From the list of arguments, we need to
        // generate two things:
        //
        // * The list of arguments the generated function needs to take.
        // * The code for processing the raw arguments and converting them to the
        //   appropriate Rust types.
        let mut args = Vec::new();
        let process_args = TokenStream::new();
        for arg in &self.args {
            unimplemented!(
                "Don't know how to generating binding for parameter {:?}",
                arg
            );
        }

        // Process the return type of the function. We need to generate two things from it:
        //
        // * The corresponding return type for the generated function.
        // * The code for processing the return type of the Rust function and converting it
        //   to the appropriate C# type.
        let ret_val = format_ident!("ret_val");
        let (return_type, process_return) = match &self.ret {
            None => (quote! { () }, TokenStream::new()),

            Some(prim) => (
                prim.to_token_stream(),
                prim.generate_return_expr(&ret_val, &mut args),
            ),
        };

        // Generate the expression for invoking the underlying Rust function.
        let orig_fn_name = format_ident!("{}", &self.ident);
        let arg_names = TokenStream::new();

        // Convert the raw list of args into a `Punctuated` so that syn/quote will handle
        // inserting commas for us.
        let args: Punctuated<_, Comma> = args.into_iter().collect();

        // Compose the various pieces to generate the final function.
        let result = quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #generated_fn_ident(#args) -> #return_type {
                use std::convert::TryInto;

                #process_args

                let #ret_val = #orig_fn_name(#arg_names);

                #process_return
            }
        };

        tokens.append_all(result);
    }
}
