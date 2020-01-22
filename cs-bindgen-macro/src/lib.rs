extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::*;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Async, Comma, RArrow},
    *,
};

#[proc_macro_attribute]
pub fn cs_bindgen(
    attr: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Create a copy of the input token stream that we can later extend with the
    // generated code. This allows us to consume the input stream without needing to
    // manually reconstruct the original input later when returning the result.
    let orig: TokenStream = tokens.clone().into();

    let input = parse_macro_input!(tokens as BindgenFn);
    dbg!(&input);

    let result = quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
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
#[derive(Debug)]
enum ReturnType {
    Default,
    Primitive(RArrow, Primitive),
    Boxed(RArrow, Box<Type>),
}

impl Parse for ReturnType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ret: syn::ReturnType = input.parse()?;
        let (arrow, inner) = match ret {
            syn::ReturnType::Default => return Ok(ReturnType::Default),
            syn::ReturnType::Type(arrow, inner) => (arrow, inner),
        };

        let ident = match &*inner {
            Type::Path(path) => match path.path.get_ident() {
                Some(ident) => ident,
                None => return Ok(ReturnType::Boxed(arrow, inner)),
            },

            _ => return Ok(ReturnType::Boxed(arrow, inner)),
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

            _ => unimplemented!("Unsupported primitive return type: {}", ident),
        };

        Ok(ReturnType::Primitive(arrow, prim))
    }
}

/// A Rust type that can be directly
#[derive(Debug)]
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

#[derive(Debug)]
struct BindgenFn {
    vis: Option<Visibility>,
    ident: Ident,
    args: Punctuated<FnArg, Comma>,
    ret: ReturnType,
}

impl Parse for BindgenFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse().ok();

        // Generate an error if the function is async.
        if let Ok(token) = input.parse::<Token![async]>() {
            return Err(syn::Error::new(
                token.span,
                "Async functions cannot be called by C# code",
            ));
        }

        input.parse::<Token![fn]>()?;
        let ident: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let args: Punctuated<FnArg, Comma> = content.parse_terminated(FnArg::parse)?;

        let ret = input.parse()?;

        // TODO: I guess this will probably break on `where` clauses?

        // NOTE: We must fully parse the body of the method in order to
        let content;
        braced!(content in input);
        let _ = content.call(Block::parse_within)?;

        Ok(BindgenFn {
            vis,
            ident,
            args,
            ret,
        })
    }
}

impl ToTokens for BindgenFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.vis;

        // Determine the game of the generated function.
        let ident = format_ident!("__cs_bindgen_generated_{}", self.ident);

        // Build the list of arguments to generated function based on the arguments of the
        // original function.
        let mut args = Vec::new();
        for arg in &self.args {
            unimplemented!(
                "Don't know how to generating binding for parameter {:?}",
                arg
            );
        }

        let ret = match &self.ret {
            ReturnType::Default => quote! { () },

            ReturnType::Boxed(..) => unimplemented!("Arbitrary return types not yet supported"),

            ReturnType::Primitive(_, prim) => {
                // If we're returning a string, we need to add an extra parameter in order
                // to be able to also return the length of the string to the calling code.
                if let Primitive::String = prim {
                    args.push(quote! {
                        out_len: *mut i32
                    });
                }

                quote! { #prim }
            }
        };

        // Convert the raw list of args into a `Punctuated` so that syn/quote will handle
        // inserting commas for us.
        let args: Punctuated<_, Comma> = args.into_iter().collect();

        let result = quote! {
            #[no_mangle]
            #vis unsafe extern "C" fn #ident(#args) -> #ret {
                unimplemented!()
            }
        };

        tokens.append_all(result);
    }
}
