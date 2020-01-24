use proc_macro2::Span;
use serde::*;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Comma,
    *,
};

mod arg;
mod primitive;
mod ret;

pub use crate::{arg::FnArg, primitive::Primitive, ret::ReturnType};

#[derive(Debug, Serialize, Deserialize)]
pub struct BindgenFn {
    ident: String,

    // TODO: Preserve variable names for function arguments or we won't be able to
    // generate code for functions that actually have args.
    pub args: Vec<FnArg>,

    pub ret: Option<Primitive>,
}

impl BindgenFn {
    /// Returns the raw string representation of the function's name.
    ///
    /// Be careful about how this function is used. If the returned value is quasi-quoted
    /// directly, it'll generate a string in the output rather than an identifier. Use
    /// `ident` to get a proper `Ident` for use in quasi-quoting, or use `format_ident!`
    /// to concatenate this value into a valid `Ident`.
    pub fn raw_ident(&self) -> &str {
        &self.ident
    }

    /// Returns the name of the function as an identifier suitable for quasi-quoting.
    pub fn ident(&self) -> Ident {
        Ident::new(&self.ident, Span::call_site())
    }

    pub fn generated_name(&self) -> String {
        format!("__cs_bindgen_generated_{}", self.ident)
    }

    pub fn generated_ident(&self) -> Ident {
        Ident::new(&self.generated_name(), Span::call_site())
    }
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
            .parse_terminated::<_, Comma>(syn::FnArg::parse)?
            .iter()
            .enumerate()
            .map(|(index, arg)| match arg {
                // Reject any functions that take some form of `self`. We'll eventually be able to
                // support these by marking entire `impl` blocks with `#[cs_bindgen]`, but for now
                // we only support free functions.
                syn::FnArg::Receiver(_) => Err(syn::Error::new(
                    arg.span(),
                    "Methods are not supported, only free functions",
                )),

                syn::FnArg::Typed(pat) => {
                    // If the argument isn't declared with a normal identifier, we construct one so
                    // that we have a valid identifier to use in the generated functions.
                    let ident = match &*pat.pat {
                        Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                        _ => format!("__arg{}", index),
                    };

                    let ty = Primitive::from_type(&pat.ty).ok_or(syn::Error::new(
                        pat.ty.span(),
                        "Unknown argument type, only primitives are supported",
                    ))?;

                    Ok(FnArg::new(ident, ty))
                }
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
