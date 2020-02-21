use cs_bindgen_shared::*;
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma, Ident};

pub fn quote_wrapper_fn<'a>(
    name: &str,
    binding: &str,
    receiver: Option<TokenStream>,
    inputs: impl Iterator<Item = (&'a str, &'a Schema)> + Clone + 'a,
    output: &Schema,
) -> TokenStream {
    // Determine the name of the wrapper function. The original function name is
    // going to be in `snake_case`, so we need to convert it to `CamelCase` to keep
    // with C# naming conventions.
    let name = format_ident!("{}", name.to_camel_case());

    let return_ty = quote_cs_type(&output);

    // Generate the declaration for the output variable and return expression. We need
    // to treat `void` returns as a special case, since C# won't let you declare values
    // with type `void` (*sigh*).
    let ret = format_ident!("__ret");
    let ret_decl = match output {
        Schema::Unit => TokenStream::default(),
        _ => quote! { #return_ty #ret; },
    };
    let ret_expr = match output {
        Schema::Unit => TokenStream::default(),
        _ => quote! { return #ret; },
    };

    // Determine if the function should be static or not based on whether or not it has
    // a receiver.
    let static_ = if receiver.is_some() {
        TokenStream::default()
    } else {
        quote! { static }
    };

    let args = quote_args(inputs.clone());
    let body = quote_wrapper_body(binding, receiver, inputs, output, &ret);

    quote! {
        public #static_ #return_ty #name(#( #args ),*)
        {
            #ret_decl
            unsafe {
                #body
            }
            #ret_expr
        }
    }
}

pub fn quote_invoke_args<'a>(
    args: impl Iterator<Item = (&'a str, &'a Schema)>,
) -> Punctuated<TokenStream, Comma> {
    args.map(|(name, schema)| {
        let ident = format_ident!("{}", name.to_mixed_case());
        match schema {
            // Basic numeric types (currently) don't require any processing.
            Schema::I8
            | Schema::I16
            | Schema::I32
            | Schema::I64
            | Schema::U8
            | Schema::U16
            | Schema::U32
            | Schema::U64
            | Schema::F32
            | Schema::F64 => ident.to_token_stream(),

            Schema::Bool => quote! { (#ident ? 1 : 0) },

            // To pass a string to Rust, we convert it into a `RawCsString` with the fixed pointer.
            // The code for wrapping the body of the function in a `fixed` block is done below,
            // since we need to generate the contents of the block first.
            Schema::String => {
                let fixed_ident = format_ident!("__fixed_{}", ident);
                quote! {
                    new RawCsString(#fixed_ident, #ident.Length)
                }
            }

            Schema::Char => todo!("Support converting a C# `char` into a Rust `char`"),

            // TODO: Add support for passing user-defined types out from Rust.
            Schema::Struct(_)
            | Schema::UnitStruct(_)
            | Schema::NewtypeStruct(_)
            | Schema::TupleStruct(_)
            | Schema::Enum(_)
            | Schema::Option(_)
            | Schema::Seq(_)
            | Schema::Tuple(_)
            | Schema::Map { .. } => todo!("Generate argument binding"),

            Schema::I128 | Schema::U128 | Schema::Unit => {
                unreachable!("Invalid argument types should have already been rejected");
            }
        }
    })
    .collect::<Punctuated<_, Comma>>()
}

pub fn quote_wrapper_body<'a>(
    binding_name: &str,
    receiver: Option<TokenStream>,
    args: impl Iterator<Item = (&'a str, &'a Schema)> + Clone,
    output: &Schema,
    ret: &Ident,
) -> TokenStream {
    // Build the list of arguments to the wrapper function and insert the receiver at
    // the beginning of the list of arguments if necessary.
    let mut invoke_args = quote_invoke_args(args.clone());
    if let Some(receiver) = receiver {
        invoke_args.insert(0, receiver);
    }

    // Construct the path the raw binding function.
    let binding = {
        let raw_ident = format_ident!("{}", binding_name);
        quote! { __bindings.#raw_ident }
    };

    // Generate the expression for invoking the raw binding and then converting the raw
    // return value into the appropriate C# type.
    let invoke = quote! { #binding(#invoke_args) };
    let invoke = match output {
        // NOTE: For `void` returns there's no intermediate variable for the return value
        // (since we can't have a `void` variable).
        Schema::Unit => quote! { #invoke; },

        // Basic numeric types (currently) don't require any processing.
        Schema::I8
        | Schema::I16
        | Schema::I32
        | Schema::I64
        | Schema::U8
        | Schema::U16
        | Schema::U32
        | Schema::U64
        | Schema::F32
        | Schema::F64 => quote! { #ret = #invoke; },

        // `bool` is returned as a `u8`, so we do an explicit comparison to convert it back
        // to a `bool` on the C# side.
        Schema::Bool => quote! { #ret = #invoke != 0; },

        // To pass a string to Rust, we convert it into a `RawCsString` with the fixed pointer.
        // The code for wrapping the body of the function in a `fixed` block is done below,
        // since we need to generate the contents of the block first.
        //
        // Once we decode the Rust string into a C# string, we also need to drop the original
        // Rust string.
        Schema::String => quote! {
            var __raw_result = #invoke;
            #ret = Encoding.UTF8.GetString(__raw_result.Ptr, (int)__raw_result.Length);
            __bindings.__cs_bindgen_drop_string(__raw_result);
        },

        Schema::Char => todo!("Support converting a C# `char` into a Rust `char`"),

        // TODO: Look up the referenced type and process the raw return value based on the
        // type of binding being generated for the type. For now we only support treating
        // named types as handles, so we always pass the handle to the type's constructor.
        Schema::Struct(output) => {
            let ty_ident = format_ident!("{}", &*output.name.name);
            quote! { #ret = new #ty_ident(#invoke); }
        }

        // TODO: Add support for passing user-defined types out from Rust.
        Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_)
        | Schema::Enum(_)
        | Schema::Option(_)
        | Schema::Seq(_)
        | Schema::Tuple(_)
        | Schema::Map { .. } => todo!("Generate argument binding"),

        Schema::I128 | Schema::U128 => {
            unreachable!("Invalid argument types should have already been rejected");
        }
    };

    // Wrap the body of the function in `fixed` blocks for any parameters that need to
    // be passed as pointers to Rust (just strings for now). We use `Iterator::fold` to
    // generate a series of nested `fixed` blocks. This is very smart code and won't be
    // hard to maintain at all, I'm sure.
    fold_fixed_blocks(invoke, args)
}

pub fn fold_fixed_blocks<'a>(
    base_invoke: TokenStream,
    args: impl Iterator<Item = (&'a str, &'a Schema)>,
) -> TokenStream {
    args.fold(base_invoke, |body, (name, schema)| match schema {
        Schema::String => {
            let arg_ident = format_ident!("{}", name.to_mixed_case());
            let fixed_ident = format_ident!("__fixed_{}", arg_ident);
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

/// Generates the argument declarations for a C# wrapper function.
///
/// Attempts to use the most idiomatic C# type that corresponds to the original type.
pub fn quote_args<'a>(
    args: impl Iterator<Item = (&'a str, &'a Schema)> + 'a,
) -> impl Iterator<Item = TokenStream> + 'a {
    args.map(|(name, schema)| {
        let ident = format_ident!("{}", name.to_mixed_case());
        let ty = quote_cs_type(schema);
        quote! { #ty #ident }
    })
}

/// Generates the idiomatic C# type corresponding to the given type schema.
pub fn quote_cs_type(schema: &Schema) -> TokenStream {
    match schema {
        // NOTE: This is only valid in a return position, it's not valid to have a `void`
        // argument. An earlier validation pass has already rejected any such cases so we
        // don't have to differentiate between the two here.
        Schema::Unit => quote! { void },

        // TODO: Should we be generating more idiomatic return types for numeric types? Any
        // numeric type that's not `int`, `long`, or `float`/double` is going to be awkward
        // to use in most cases.
        //
        // Tracking issue: https://github.com/randomPoison/cs-bindgen/issues/4
        Schema::I8 => quote! { sbyte },
        Schema::I16 => quote! { short },
        Schema::I32 => quote! { int },
        Schema::I64 => quote! { long },
        Schema::U8 => quote! { byte },
        Schema::U16 => quote! { ushort },
        Schema::U32 => quote! { uint },
        Schema::U64 => quote! { ulong },
        Schema::F32 => quote! { float },
        Schema::F64 => quote! { double },
        Schema::Bool => quote! { byte },
        Schema::String => quote! { string },

        Schema::Char => todo!("Support passing single chars"),

        Schema::Struct(struct_) => format_ident!("{}", &*struct_.name.name).to_token_stream(),

        // TODO: Add support for passing user-defined types out from Rust.
        Schema::UnitStruct(_)
        | Schema::NewtypeStruct(_)
        | Schema::TupleStruct(_)
        | Schema::Enum(_)
        | Schema::Option(_)
        | Schema::Seq(_)
        | Schema::Tuple(_)
        | Schema::Map { .. } => todo!("Generate argument binding"),

        Schema::I128 | Schema::U128 => {
            unreachable!("Invalid argument types should have already been rejected");
        }
    }
}
