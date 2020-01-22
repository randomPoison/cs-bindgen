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

#[derive(Debug)]
struct BindgenFn {
    args: Punctuated<FnArg, Comma>,
    ret: ReturnType,
}

impl Parse for BindgenFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let asyncness = input.parse::<Token![async]>().ok();
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

        Ok(BindgenFn { args, ret })
    }
}
