use crate::{parse_signature, FnArg, ReturnType};
use serde::*;
use syn::{spanned::Spanned, Error, ImplItem, ImplItemMethod, Type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindgenImpl {
    pub ty_ident: String,
    pub methods: Vec<Method>,
}

impl BindgenImpl {
    pub fn from_item(item: syn::ItemImpl) -> syn::Result<Self> {
        if let Some((_, path, _)) = item.trait_ {
            return Err(Error::new(
                path.span(),
                "Trait impls are not yet supported with `#[cs_bindgen]`",
            ));
        }

        if !item.generics.params.is_empty() {
            return Err(Error::new(
                item.generics.span(),
                "Generic impls are not not supported with `#[cs_bindgen]`",
            ));
        }

        let ty_ident = if let Type::Path(path) = *item.self_ty {
            path.path
                .get_ident()
                .map(|ident| ident.to_string())
                .ok_or(Error::new(
                    path.span(),
                    "Self type not supported in impl for `#[cs_bindgen]`",
                ))?
        } else {
            return Err(Error::new(
                item.self_ty.span(),
                "Impls for this type of item are not supported by `#[cs_bindgen]`",
            ));
        };

        let methods = item
            .items
            .into_iter()
            .filter_map(|item| match item {
                ImplItem::Method(item) => Some(item),
                _ => None,
            })
            .map(Method::from_item)
            .collect::<syn::Result<_>>()?;

        Ok(Self { ty_ident, methods })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    ident: String,

    receiver: Receiver,

    // TODO: Preserve variable names for function arguments or we won't be able to
    // generate code for functions that actually have args.
    pub args: Vec<FnArg>,

    pub ret: ReturnType,
}

impl Method {
    pub fn from_item(item: ImplItemMethod) -> syn::Result<Self> {
        let (ident, receiver, args, ret) = parse_signature(&item.sig)?;

        Ok(Method {
            ident,
            receiver: receiver.ok_or(Error::new(
                item.sig.span(),
                "Invalid receiver for `#[cs_bindgen]` method",
            ))?,
            args,
            ret,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Receiver {
    Ref,
    RefMut,
    Value,
}

impl Receiver {
    pub fn from_syn(arg: &syn::FnArg) -> syn::Result<Self> {
        match arg {
            syn::FnArg::Receiver(recv) => Ok({
                if recv.reference.is_some() {
                    if recv.mutability.is_some() {
                        Receiver::RefMut
                    } else {
                        Receiver::Ref
                    }
                } else {
                    Receiver::Value
                }
            }),

            syn::FnArg::Typed(_) => Err(Error::new(
                arg.span(),
                "Invalid receiver for `#[cs_bindgen]` method",
            )),
        }
    }
}
