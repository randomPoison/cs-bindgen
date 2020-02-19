extern crate proc_macro;

use crate::func::*;
use proc_macro2::TokenStream;
use quote::*;
use syn::{punctuated::Punctuated, token::Comma, *};

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

mod func;

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
        Item::Impl(item) => quote_impl_item(item),

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
    reject_generics(
        &signature.generics,
        "Generic functions not supported with `#[cs_bindgen]`",
    )?;

    // Determine the name of the generated function.
    let ident = signature.ident;
    let binding_ident = format_binding_ident!(ident);

    // Process the arguments to the function.
    let inputs = extract_inputs(signature.inputs)?;
    let binding_inputs = quote_binding_inputs(&inputs);
    let convert_inputs = quote_input_conversion(&inputs);

    // Normalize the return type of the function.
    let return_type = normalize_return_type(&signature.output);

    // Generate the list of argument names. Used both for forwarding arguments into the
    // original function, and for populating the metadata item.
    let arg_names = inputs.iter().map(|(ident, _)| ident);

    // Compose the various pieces together into the final binding function.
    let invoke_expr = quote! { #ident(#( #arg_names, )*) };
    let binding = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #binding_ident(
            #( #binding_inputs, )*
        ) -> <#return_type as cs_bindgen::abi::IntoAbi>::Abi {
            #( #convert_inputs )*
            cs_bindgen::abi::IntoAbi::into_abi(#invoke_expr)
        }
    };

    // Generate the name of the describe function.
    let describe_ident = format_describe_ident!(ident);

    // Generate string versions of the two function idents.
    let name = ident.to_string();
    let binding_name = binding_ident.to_string();

    let describe_args = inputs.iter().map(|(ident, ty)| {
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
    reject_generics(
        &item.generics,
        "Generic structs are not supported with `#[cs_bindgen]`",
    )?;

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

fn quote_impl_item(item: ItemImpl) -> syn::Result<TokenStream> {
    // Generate an error for any generic parameters.
    reject_generics(
        &item.generics,
        "Generic `impl` blocks are not supported with `#[cs_bindgen]`",
    )?;

    // Generate an error for trait impls. Only inherent impls are allowed for now.
    if let Some((_, trait_, _)) = item.trait_ {
        return Err(Error::new_spanned(
            trait_,
            "Trait impls not supported with `#[cs_bindgen]`",
        ));
    }

    let self_ty = item.self_ty;

    // Iterate over the items declared in the impl block and generate bindings for any
    // supported item types.
    item.items
        .into_iter()
        .filter_map(|item| {
            match item {
                ImplItem::Method(item) => Some(quote_method_item(item, &self_ty)),

                // Ignore all other unsupported associated item types. We don't generate bindings
                // for them, but it's otherwise not an error to include them in an `impl` block
                // tagged with `#[cs_bindgen]`.
                _ => None,
            }
        })
        .collect::<syn::Result<TokenStream>>()
}

fn quote_method_item(item: ImplItemMethod, self_ty: &Type) -> syn::Result<TokenStream> {
    // TODO: Generate binding function.

    // Extract the signature, which contains the bulk of the information we care about.
    let signature = item.sig;

    // Generate an error for any generic parameters.
    reject_generics(
        &signature.generics,
        "Generic functions not supported with `#[cs_bindgen]`",
    )?;

    // Process the receiver for the method, if any.
    let receiver_ident = quote! { self_ };
    let mut binding_inputs = Punctuated::<_, Comma>::new();
    let mut convert_inputs = Vec::new();
    let mut arg_names = Punctuated::<_, Comma>::new();
    if let Some(arg) = signature.receiver() {
        let self_ty = match arg {
            // Expand the full self type based on how the receiver was declared:
            //
            // * `self` -> `self_ty`
            // * `&self` -> `&self_ty`
            // * `&mut self` -> `&mut self_ty`
            FnArg::Receiver(arg) => {
                if arg.reference.is_some() {
                    if arg.mutability.is_some() {
                        quote! { &mut #self_ty }
                    } else {
                        quote! { & #self_ty }
                    }
                } else {
                    self_ty.to_token_stream()
                }
            }

            // If the method was declared using an arbitrary self type (e.g. `self: Foo`), directly
            // used the declared type.
            //
            // TODO: There's likely some extra work needed here in order to fully support arbitrary
            // self types: While the macro won't generate an error, we're probably not going to
            // generate the ideal bindings in all cases.
            FnArg::Typed(arg) => arg.ty.to_token_stream(),
        };

        // Generate the necessary declarations for the method receiver.
        binding_inputs.push(quote! { #receiver_ident: #self_ty });
        convert_inputs.push(quote! {
            let #receiver_ident = cs_bindgen::abi::FromAbi::from_abi(#receiver_ident);
        });
        arg_names.push(receiver_ident.clone());
    }

    // Determine the name of the generated function.
    let ident = signature.ident;
    let binding_ident = format_binding_ident!(ident);

    // Process the arguments to the function.
    let inputs = extract_inputs(signature.inputs)?;
    binding_inputs.extend(quote_binding_inputs(&inputs));
    convert_inputs.extend(quote_input_conversion(&inputs));

    // Generate the list of argument names. Used both for forwarding arguments into the
    // original function, and for populating the metadata item.
    arg_names.extend(inputs.iter().map(|(ident, _)| ident.to_token_stream()));

    // Normalize the return type of the function.
    let return_type = normalize_return_type(&signature.output);

    // Compose the various pieces together into the final binding function.
    let binding = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #binding_ident(
            #binding_inputs
        ) -> <#return_type as cs_bindgen::abi::IntoAbi>::Abi {

            #( #convert_inputs )*
            cs_bindgen::abi::IntoAbi::into_abi(#self_ty::#ident(#arg_names))
        }
    };

    // TODO: Generate descriptor function.

    Ok(quote! {
        #binding
        // #describe
    })
}
