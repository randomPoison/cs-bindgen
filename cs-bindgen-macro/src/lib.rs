extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::*;
use syn::*;

#[proc_macro_attribute]
pub fn cs_bindgen(
    _attr: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Create a copy of the input token stream that we can later extend with the
    // generated code. This allows us to consume the input stream without needing to
    // manually reconstruct the original input later when returning the result.
    let mut result: TokenStream = tokens.clone().into();

    let generated = match parse_macro_input!(tokens as Item) {
        Item::Fn(item) => quote_fn_item(item),
        Item::Struct(_item) => Ok(quote! {}),
        Item::Impl(_item) => Ok(quote! {}),

        // Generate an error for any unknown item types.
        item @ _ => Err(Error::new_spanned(
            item,
            "Item not supported with `#[cs_bindgen]`",
        )),
    }
    .unwrap_or_else(|err| err.to_compile_error());

    // Append the generated binding and declaration to the result stream.
    result.extend(generated);

    result.into()
}

fn quote_fn_item(item: ItemFn) -> syn::Result<TokenStream> {
    // Extract the signature, which contains the bulk of the information we care about.
    let signature = item.sig;

    // Generate an error for any generic parameters.
    let generics = signature.generics;
    let has_generics = generics.type_params().next().is_some()
        || generics.lifetimes().next().is_some()
        || generics.const_params().next().is_some();
    if has_generics {
        return Err(Error::new_spanned(
            generics,
            "Generic functions not supported with `#[cs_bindgen]`",
        ));
    }

    // Determine the name of the generated function.
    let ident = signature.ident;
    let binding_ident = format_ident!("__cs_bindgen_generated__{}", ident);

    let args: Vec<(Ident, Box<Type>)> = signature
        .inputs
        .iter()
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
            let ident = match &*arg.pat {
                Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                _ => format_ident!("__arg{}", index),
            };

            Ok((ident, arg.ty.clone()))
        })
        .collect::<Result<_>>()?;

    // Process the arguments to the function. From the list of arguments, we need to
    // generate two things:
    //
    // * The list of arguments the generated function needs to take.
    // * The code for processing the raw arguments and converting them to the
    //   appropriate Rust types.
    let binding_args = args.iter().map(|(ident, ty)| {
        quote! {
            #ident: <#ty as cs_bindgen::abi::FromAbi>::Abi
        }
    });
    let process_args = args.iter().map(|(ident, _)| {
        quote! {
            let #ident = cs_bindgen::abi::FromAbi::from_abi(#ident);
        }
    });

    let return_type = match signature.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => ty.to_token_stream(),
    };

    // Generate the list of argument names. Used both for forwarding arguments into the
    // original function, and for populating the metadata item.
    let arg_names = args.iter().map(|(ident, _)| ident);

    // Compose the various pieces together into the final binding function.
    let invoke_expr = quote! { #ident(#( #arg_names, )*) };
    let binding = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #binding_ident(
            #( #binding_args, )*
        ) -> <#return_type as cs_bindgen::abi::IntoAbi>::Abi
    {
            #( #process_args )*
            cs_bindgen::abi::IntoAbi::into_abi(#invoke_expr)
        }
    };

    // Generate the name of the describe function.
    let describe_ident = format_ident!("__cs_bindgen_describe__{}", ident);

    // Generate string versions of the two function idents.
    let name = ident.to_string();
    let binding_name = binding_ident.to_string();

    let describe_args = args.iter().map(|(ident, ty)| {
        let name = ident.to_string();
        quote! {
            (#name.into(), encode::<#ty>().expect("Failed to generate schema for argument type"))
        }
    });

    // Generate the describe function.
    let describe = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> Box<cs_bindgen::abi::RawVec<u8>> {
            use cs_bindgen::shared::{schematic::encode, Func};

            let export = Func {
                name: #name.into(),
                binding: #binding_name.into(),
                inputs: vec![#(
                    #describe_args,
                )*],
                output: encode::<#return_type>().expect("Failed to generate schema for return type"),
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }
    };

    Ok(quote! {
        #binding
        #describe
    })
}

/*
fn build_function_args(
    args: &[FnArg],
    arg_decls: &mut Punctuated<TokenStream, Comma>,
    process_args: &mut TokenStream,
) {
    for arg in args {
        let ident = arg.ident();
        let ty = match arg.ty {
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
        };

        arg_decls.push(quote! { #ident: #ty });

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
}

fn quote_primitive_return_type(prim: Primitive) -> TokenStream {
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

fn quote_method(item: &Method) -> TokenStream {
    let ty_ident = item.strct.ident();
    let receiver_arg_ident = format_ident!("self_");

    // Determine the name of the generated function.
    let generated_fn_ident = item.binding_ident();

    let mut args = Punctuated::<_, Comma>::new();
    let mut process_args = TokenStream::default();
    let mut arg_names = Punctuated::<_, Comma>::new();

    // Generate bindings for the method receiver (i.e. the `self` argument).
    if item.method.receiver.is_some() {
        args.push(quote! {
            #receiver_arg_ident: &std::sync::Mutex<#ty_ident>
        });

        process_args.extend(quote! {
            let mut #receiver_arg_ident = #receiver_arg_ident.lock().expect("Handle mutex was poisoned");
            let #receiver_arg_ident = &mut *#receiver_arg_ident;
        });

        arg_names.push(receiver_arg_ident.clone());
    }

    // Process the remaining arguments.
    build_function_args(&item.method.args, &mut args, &mut process_args);

    // Process the return value.
    let ret_val = format_ident!("ret_val");
    let (return_type, process_return) = match item.method.ret {
        ReturnType::Default => (quote! { () }, TokenStream::new()),

        ReturnType::SelfType => {
            let ty = quote! {
                std::boxed::Box<std::sync::Mutex<#ty_ident>>
            };

            let process = quote! {
                std::boxed::Box::new(std::sync::Mutex::new(#ret_val))
            };
            (ty, process)
        }

        ReturnType::Primitive(prim) => (
            quote_primitive_return_type(prim),
            quote_return_expr(prim, &ret_val),
        ),
    };

    // Generate the expression for invoking the underlying Rust function.
    let method_name = item.method.ident();
    let orig_fn = quote! { #ty_ident::#method_name };

    arg_names.extend(item.method.args.iter().map(FnArg::ident));

    // Compose the various pieces to generate the final function.
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #generated_fn_ident(#args) -> #return_type {
            use std::convert::TryInto;

            #process_args

            let #ret_val = #orig_fn(#arg_names);

            #process_return
        }
    }
}

fn quote_drop_fn(item: &Struct) -> TokenStream {
    let ty_ident = item.ident();
    let ident = item.drop_fn_ident();
    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #ident(_: std::boxed::Box<std::sync::Mutex<#ty_ident>>) {}
    }
}
*/
