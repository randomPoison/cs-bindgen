use crate::{enumeration::*, func::*, strukt::*};
use proc_macro2::TokenStream;
use quote::*;
use std::fmt::Display;
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

mod enumeration;
mod func;
mod handle;
mod strukt;
mod value;

#[proc_macro_attribute]
pub fn cs_bindgen(
    _attr: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Create a copy of the input token stream that we can later extend with the
    // generated code. This allows us to consume the input stream without needing to
    // manually reconstruct the original input later when returning the result.
    let mut result: TokenStream = tokens.clone().into();

    // Generate the bindings for the annotated item, or generate an error if the
    // item/attribute is invalid.
    let generated = match parse_macro_input!(tokens as Item) {
        Item::Fn(item) => quote_fn_item(item),
        Item::Struct(item) => quote_struct_item(item),
        Item::Impl(item) => quote_impl_item(item),
        Item::Enum(item) => quote_enum_item(item),

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BindingStyle {
    Handle,
    Value,
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
    let binding_inputs = inputs
        .iter()
        .map(|(ident, ty)| quote_binding_inputs(ident, ty));
    let convert_inputs = inputs
        .iter()
        .map(|(ident, _)| quote_input_conversion(ident));

    // Generate the output portion of the binding function declaration.
    let return_decl = match &signature.output {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, return_type) => quote! {
            -> <#return_type as cs_bindgen::abi::Abi>::Abi
        },
    };

    // Generate the expression for describing the output of the function.
    let describe_output = match &signature.output {
        ReturnType::Default => quote! { None },
        ReturnType::Type(_, return_type) => quote! {
            Some(<#return_type as cs_bindgen::abi::Abi>::repr())
        },
    };

    // Generate the list of argument names. Used both for forwarding arguments into the
    // original function, and for populating the metadata item.
    let arg_names = inputs.iter().map(|(ident, _)| ident);

    let invoke_expr = quote! { #ident(#( #arg_names, )*) };
    let return_expr = match &signature.output {
        ReturnType::Default => invoke_expr,
        ReturnType::Type(..) => quote! { cs_bindgen::abi::Abi::into_abi(#invoke_expr) },
    };

    // Compose the various pieces together into the final binding function.
    let binding = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #binding_ident(
            #( #binding_inputs, )*
        ) #return_decl {
            #( #convert_inputs )*
            #return_expr
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
            cs_bindgen::shared::FnArg::new(#name, <#ty as cs_bindgen::abi::Abi>::repr())
        }
    });

    // Generate the describe function.
    let describe = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> Box<cs_bindgen::abi::RawString> {
            use cs_bindgen::shared::{schematic::describe, Func};

            let export = Func {
                name: #name.into(),
                binding: #binding_name.into(),
                inputs: vec![#(
                    #describe_args,
                )*],
                output: #describe_output,
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }
    };

    Ok(quote! {
        #binding
        #describe
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
    // Generate the binding function
    // =============================

    // Extract the signature, which contains the bulk of the information we care about.
    let signature = item.sig;

    // Generate an error for any generic parameters.
    reject_generics(
        &signature.generics,
        "Generic functions not supported with `#[cs_bindgen]`",
    )?;

    // Process the receiver for the method, if any:
    //
    // * For the binding function, we need to add the additional input to the list of
    //   inputs.
    // * For the descriptor function, we need generate the value of the `receiver` field
    //   on the created `Method` object.
    //
    // TODO: Rewrite all this it's very bad and super hard to follow. Probably the thing
    // to do would be to first parse out the receiver style as an enum, then do a
    // separate `match` on it for each of the values we want to generate.
    let (mut binding_args, describe_receiver) = match signature.receiver() {
        Some(arg) => {
            let (self_ty, describe) = match arg {
                // Expand the full self type based on how the receiver was declared:
                //
                // * `self` -> `self_ty`
                // * `&self` -> `&self_ty`
                // * `&mut self` -> `&mut self_ty`
                FnArg::Receiver(arg) => {
                    if arg.reference.is_some() {
                        if arg.mutability.is_some() {
                            (
                                quote! { &mut #self_ty },
                                quote! { Some(ReceiverStyle::RefMut) },
                            )
                        } else {
                            (quote! { & #self_ty }, quote! { Some(ReceiverStyle::Ref) })
                        }
                    } else {
                        (
                            self_ty.to_token_stream(),
                            quote! { Some(ReceiverStyle::Move) },
                        )
                    }
                }

                // If the method was declared using an arbitrary self type (e.g. `self: Foo`), directly
                // used the declared type.
                //
                // TODO: There's likely some extra work needed here in order to fully support arbitrary
                // self types: While the macro won't generate an error, we're probably not going to
                // generate the ideal bindings in all cases.
                //
                // We probably want to treat arbitrary self type functions more like static functions
                // in C# than methods. So maybe convert it to a regular function with a normal self type,
                // i.e. treat it as if there were no receiver?
                FnArg::Typed(arg) => (arg.ty.to_token_stream(), quote! { None }),
            };

            (vec![(format_ident!("self_"), self_ty)], describe)
        }

        None => (Default::default(), quote! { None }),
    };

    // Determine the name of the generated function.
    let ident = signature.ident;
    let self_ident = extract_type_ident(self_ty)?;
    let mangled_name = format!("{}__{}", ident, self_ident);
    let binding_ident = format_binding_ident!(mangled_name);

    // Process the arguments to the function.
    let inputs = extract_inputs(signature.inputs)?;
    binding_args.extend(
        inputs
            .iter()
            .map(|(ident, ty)| (ident.clone(), ty.into_token_stream())),
    );
    let binding_inputs = binding_args
        .iter()
        .map(|(ident, ty)| quote_binding_inputs(ident, ty));
    let convert_inputs = binding_args
        .iter()
        .map(|(ident, _)| quote_input_conversion(ident));

    // Generate the list of argument names. Used both for forwarding arguments into the
    // original function, and for populating the metadata item.
    let arg_names = binding_args
        .iter()
        .map(|(ident, _)| ident.to_token_stream());

    // Generate the output portion of the binding function declaration.
    let return_decl = match &signature.output {
        ReturnType::Default => quote! {},
        ReturnType::Type(_, return_type) => quote! {
            -> <#return_type as cs_bindgen::abi::Abi>::Abi
        },
    };

    // Generate the expression for describing the output of the function.
    let describe_output = match &signature.output {
        ReturnType::Default => quote! { None },
        ReturnType::Type(_, return_type) => quote! {
            Some(<#return_type as cs_bindgen::abi::Abi>::repr())
        },
    };

    let invoke = quote! { #self_ty::#ident(#( #arg_names, )*) };
    let return_expr = match &signature.output {
        ReturnType::Default => invoke,
        ReturnType::Type(..) => quote! { cs_bindgen::abi::Abi::into_abi(#invoke) },
    };

    // Compose the various pieces together into the final binding function.
    let binding = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #binding_ident(
            #( #binding_inputs, )*
        ) #return_decl {
            #( #convert_inputs )*
            #return_expr
        }
    };

    // Generate the describe function.
    // ===============================

    // Generate the name of the describe function.
    let describe_ident = format_describe_ident!(mangled_name);

    // Generate string versions of the two function idents.
    let name = ident.to_string();
    let binding_name = binding_ident.to_string();

    let describe_args = inputs.iter().map(|(ident, ty)| {
        let name = ident.to_string();
        quote! {
            cs_bindgen::shared::FnArg::new(#name, <#ty as cs_bindgen::abi::Abi>::repr())
        }
    });

    let describe = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> Box<cs_bindgen::abi::RawString> {
            use cs_bindgen::shared::{schematic::describe, Method, ReceiverStyle};

            let export = Method {
                name: #name.into(),
                binding: #binding_name.into(),
                self_type: <#self_ty as cs_bindgen::shared::Named>::type_name(),
                receiver: #describe_receiver,
                inputs: vec![#(
                    #describe_args,
                )*],
                output: #describe_output,
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }
    };

    Ok(quote! {
        #binding
        #describe
    })
}

/// Returns `true` if any of the specified attributes are a `derive()` containing `Copy`.
fn has_derive_copy(attributes: &[Attribute]) -> syn::Result<bool> {
    // Get the `#[derive(..)]` attribute, or return `false` if none is present.
    let attr = match attributes.iter().find(|attr| {
        attr.path
            .get_ident()
            .map(|ident| ident == "derive")
            .unwrap_or(false)
    }) {
        Some(attr) => attr.parse_meta()?,
        None => return Ok(false),
    };

    let list = match attr {
        Meta::List(list) => list,
        _ => return Ok(false),
    };

    Ok(list.nested.into_iter().any(|nested| {
        let nested = match nested {
            NestedMeta::Meta(meta) => meta,
            _ => return false,
        };

        let path = match nested {
            Meta::Path(path) => path,
            _ => return false,
        };

        // TODO: Handle the case where the user specified the full path for the trait, i.e.
        // `std::marker::Copy`.
        path.get_ident()
            .map(|ident| ident == "Copy")
            .unwrap_or(false)
    }))
}

/// Generates an error if any generic parameters are present.
///
/// In general we can't support `#[cs_bindgen]` on generic items, any item that
/// supports generic parameters needs to generate an error during parsing. This
/// helper method can be used to check the `Generics` AST node that syn generates
/// and will return an error if the node contains any generic parameters.
fn reject_generics<M: Display>(generics: &Generics, message: M) -> syn::Result<()> {
    let has_generics = generics.type_params().next().is_some()
        || generics.lifetimes().next().is_some()
        || generics.const_params().next().is_some();
    if has_generics {
        Err(Error::new_spanned(generics, message))
    } else {
        Ok(())
    }
}

fn describe_named_type(ident: &Ident, style: BindingStyle) -> TokenStream {
    let describe_ident = format_describe_ident!(ident);
    let index_fn = index_fn_ident(ident).to_string();
    let drop_vec_fn = drop_vec_fn_ident(ident).to_string();

    let style = match style {
        BindingStyle::Handle => quote! {
            Handle
        },

        BindingStyle::Value => quote! {
            Value(cs_bindgen::shared::schematic::describe::<#ident>())
        },
    };

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #describe_ident() -> std::boxed::Box<cs_bindgen::abi::RawString> {
            // NOTE: We need to import `schematic` so that usage of the `type_name!` macro
            // resolves correctly, since the expanded code references `schematic` directly.
            use cs_bindgen::shared::schematic;

            let export = cs_bindgen::shared::NamedType {
                type_name: <#ident as cs_bindgen::shared::Named>::type_name(),
                binding_style: cs_bindgen::shared::BindingStyle::#style,
                index_fn: #index_fn.into(),
                drop_vec_fn: #drop_vec_fn.into(),
            };

            std::boxed::Box::new(cs_bindgen::shared::serialize_export(export).into())
        }
    }
}

/// Generates an impl of `Named` for the specified type.
fn impl_named(ident: &Ident) -> TokenStream {
    quote! {
        impl cs_bindgen::shared::Named for #ident {
            fn type_name() -> cs_bindgen::shared::TypeName {
                cs_bindgen::shared::TypeName::new(stringify!(#ident), module_path!())
            }
        }
    }
}

/// Generates an impl of the `repr` function in the `Abi` trait for the specified
/// type.
fn repr_impl(ident: &Ident) -> TokenStream {
    quote! {
        fn repr() -> cs_bindgen::shared::Repr {
            cs_bindgen::shared::Repr::named::<#ident>()
        }
    }
}

/// Generates a valid identifier from the given type.
///
/// Returns an error if the type is not a `Type::Path`. Otherwise, the segments of
/// the path are concatenated with `__` to create an identifier from the type
/// reference.
fn extract_type_ident(ty: &Type) -> syn::Result<Ident> {
    let path = match ty {
        Type::Path(path) => path,
        _ => return Err(Error::new_spanned(ty, "Unsupported type expression, only type paths are supported with `#[cs_bindgen]`, e.g. `Foo` or `foo::bar::Baz`")),
    };

    let ident_string = path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("__");
    Ok(format_ident!("{}", ident_string))
}

fn index_fn_ident(ty: &Ident) -> Ident {
    format_ident!("__cs_bindgen_generated_index_{}", ty)
}

/// Generates a function for converting an element in a slice.
fn quote_index_fn(ty: &Ident) -> TokenStream {
    let fn_ident = index_fn_ident(ty);
    quote! {
        #[no_mangle]
        #[allow(bad_style)]
        pub unsafe extern "C" fn #fn_ident(
            slice: cs_bindgen::abi::RawSlice<#ty>,
            index: usize,
        ) -> <#ty as cs_bindgen::abi::Abi>::Abi {
            slice.convert_element(index)
        }
    }
}

fn drop_vec_fn_ident(ty: &Ident) -> Ident {
    format_ident!("__cs_bindgen_generated_drop_vec_{}", ty)
}

fn quote_vec_drop_fn(ty: &Ident) -> TokenStream {
    let fn_ident = drop_vec_fn_ident(ty);
    quote! {
        #[no_mangle]
        #[allow(bad_style)]
        pub unsafe extern "C" fn #fn_ident(raw: cs_bindgen::abi::RawVec<#ty>) {
            let _ = raw.into_vec();
        }
    }
}
