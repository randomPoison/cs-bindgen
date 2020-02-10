extern crate proc_macro;

use cs_bindgen_shared::{FnArg, *};
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma, *};

#[proc_macro_attribute]
pub fn cs_bindgen(
    _attr: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Create a copy of the input token stream that we can later extend with the
    // generated code. This allows us to consume the input stream without needing to
    // manually reconstruct the original input later when returning the result.
    let orig: TokenStream = tokens.clone().into();

    let item = parse_macro_input!(tokens as BindgenItem);
    let quoted = match &item {
        BindgenItem::Fn(input) => quote_bindgen_fn(input),
        BindgenItem::Struct(_) => todo!("Generate binding code for structs"),
        BindgenItem::Impl(_) => todo!("Generate binding code for impls"),
    };

    // Serialize the parsed function declaration into JSON so that it can be stored in
    // a variable in the generated WASM module.
    let decl_json = serde_json::to_string(&item).expect("Failed to serialize decl to JSON");
    let decl_var_ident = format_ident!("__cs_bindgen_decl_json_{}", &*item.raw_ident());
    let decl_ptr_ident = format_ident!("__cs_bindgen_decl_ptr_{}", &*item.raw_ident());
    let decl_len_ident = format_ident!("__cs_bindgen_decl_len_{}", &*item.raw_ident());

    let result = quote! {
        #orig

        #quoted

        #[allow(bad_style)]
        static #decl_var_ident: &str = #decl_json;

        #[no_mangle]
        pub extern "C" fn #decl_ptr_ident() -> *const u8 {
            #decl_var_ident.as_ptr()
        }

        #[no_mangle]
        pub extern "C" fn #decl_len_ident() -> usize {
            #decl_var_ident.len()
        }
    };

    result.into()
}

fn quote_bindgen_fn(bindgen_fn: &BindgenFn) -> TokenStream {
    // Determine the name of the generated function.
    let generated_fn_ident = bindgen_fn.generated_ident();

    // Process the arguments to the function. From the list of arguments, we need to
    // generate two things:
    //
    // * The list of arguments the generated function needs to take.
    // * The code for processing the raw arguments and converting them to the
    //   appropriate Rust types.
    let mut args = Vec::new();
    let mut process_args = TokenStream::new();
    for arg in &bindgen_fn.args {
        let ident = arg.ident();
        let ty = quote_primitive_arg_type(arg.ty);
        args.push(quote! { #ident: #ty });

        match arg.ty {
            // Strings are passed in as utf-16 arrays (specifically as a `RawCsString`), so we
            // convert the data into a `String`.
            Primitive::String => process_args.append_all(quote! {
                let #ident = #ident.into_string();
            }),

            // Bools are passed in as a `u8`, so we need to re-bind the variable as a `bool` by
            // explicitly checking the value.
            Primitive::Bool => process_args.append_all(quote! {
                let #ident = #ident != 0;
            }),

            // The remaining primitive types don't require any additional processing.
            _ => {}
        }
    }

    let arg_names = bindgen_fn
        .args
        .iter()
        .map(FnArg::ident)
        .collect::<Punctuated<_, Comma>>();

    // Process the return type of the function. We need to generate two things from it:
    //
    // * The corresponding return type for the generated function.
    // * The code for processing the return type of the Rust function and converting it
    //   to the appropriate C# type.
    let ret_val = format_ident!("ret_val");
    let (return_type, process_return) = match bindgen_fn.ret.primitive() {
        None => (quote! { () }, TokenStream::new()),

        Some(prim) => (quote_raw_primitive(prim), quote_return_expr(prim, &ret_val)),
    };

    // Generate the expression for invoking the underlying Rust function.
    let orig_fn_name = format_ident!("{}", &bindgen_fn.raw_ident());

    // Convert the raw list of args into a `Punctuated` so that syn/quote will handle
    // inserting commas for us.
    let args: Punctuated<_, Comma> = args.into_iter().collect();

    // Compose the various pieces to generate the final function.
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #generated_fn_ident(#args) -> #return_type {
            use std::convert::TryInto;

            #process_args

            let #ret_val = #orig_fn_name(#arg_names);

            #process_return
        }
    }
}

fn quote_primitive_arg_type(prim: Primitive) -> TokenStream {
    match prim {
        Primitive::String => quote! { cs_bindgen::RawCsString },
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
    }
}

fn quote_raw_primitive(prim: Primitive) -> TokenStream {
    match prim {
        Primitive::String => quote! { cs_bindgen::RawString },
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
    }
}

/// Generates the code for returning the final result of the function.
fn quote_return_expr(prim: Primitive, ret_val: &Ident) -> TokenStream {
    match prim {
        // Convert the `String` into a `RawString`.
        Primitive::String => quote! {
            #ret_val.into()
        },

        // Cast the bool to a `u8` in order to pass it to C# as a numeric value.
        Primitive::Bool => quote! {
            #ret_val as u8
        },

        // All other primitive types are ABI-compatible with a corresponding C# type, and
        // require no extra processing to be returned.
        _ => quote! { #ret_val },
    }
}
