extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::*;
use syn::*;

macro_rules! format_binding_ident {
    ($ident:expr) => {
        format_ident!("__cs_bindgen_generated__{}", $ident);
    };
}

macro_rules! format_describe_ident {
    ($ident:expr) => {
        format_ident!("__cs_bindgen_describe__{}", $ident);
    };
}

macro_rules! format_drop_ident {
    ($ident:expr) => {
        format_ident!("__cs_bindgen_drop__{}", $ident);
    };
}

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
        Item::Struct(item) => quote_struct_item(item),
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
    let binding_ident = format_binding_ident!(ident);

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
    let describe_ident = format_describe_ident!(ident);

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
        pub unsafe extern "C" fn #describe_ident() -> Box<cs_bindgen::abi::RawString> {
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

fn quote_struct_item(item: ItemStruct) -> syn::Result<TokenStream> {
    let ident = item.ident;
    let describe_ident = format_describe_ident!(ident);
    let drop_ident = format_drop_ident!(ident);

    Ok(quote! {
        // Implement `Encode` for the exported type.
        impl cs_bindgen::shared::schematic::Encode for #ident {
            fn encode<E>(encoder: E) -> Result<E::Ok, E::Error>
            where
                E: cs_bindgen::shared::schematic::Encoder,
            {
                encoder.encode_struct(cs_bindgen::shared::schematic::type_name!(#ident))
            }
        }

        // Implement `From/IntoAbi` conversions for the type and references to the type.

        impl cs_bindgen::abi::IntoAbi for #ident {
            type Abi = std::boxed::Box<Self>;

            fn into_abi(self) -> Self::Abi {
                std::boxed::Box::new(self)
            }
        }

        impl cs_bindgen::abi::FromAbi for #ident {
            type Abi = std::boxed::Box<Self>;

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                *abi
            }
        }


        impl<'a> cs_bindgen::abi::IntoAbi for &'a #ident {
            type Abi = Self;

            fn into_abi(self) -> Self::Abi {
                self
            }
        }

        impl<'a> cs_bindgen::abi::FromAbi for &'a #ident {
            type Abi = Self;

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                abi
            }
        }

        impl<'a> cs_bindgen::abi::IntoAbi for &'a mut #ident {
            type Abi = Self;

            fn into_abi(self) -> Self::Abi {
                self
            }
        }

        impl<'a> cs_bindgen::abi::FromAbi for &'a mut #ident {
            type Abi = Self;

            unsafe fn from_abi(abi: Self::Abi) -> Self {
                abi
            }
        }

        // Export a function that describes the exported type.
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> std::boxed::Box<cs_bindgen::abi::RawString> {
            use cs_bindgen::shared::{schematic::{encode, type_name}, Func};

            // TODO: Use the `Encode` impl for the type, rather than constructing the schema
            // directly.
            let export = cs_bindgen::shared::schematic::Struct {
                name: type_name!(#ident),
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }

        // Export a function that can be used for dropping an instance of the type.
        #[no_mangle]
        pub unsafe extern "C" fn #drop_ident(_: <#ident as cs_bindgen::abi::FromAbi>::Abi) {}
    })
}

// fn quote_drop_fn(item: &Struct) -> TokenStream {
//     let ty_ident = item.ident();
//     let ident = item.drop_fn_ident();
//     quote! {
//         #[no_mangle]
//         pub unsafe extern "C" fn #ident(_: std::boxed::Box<std::sync::Mutex<#ty_ident>>) {}
//     }
// }
