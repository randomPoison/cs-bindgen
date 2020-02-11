use crate::{BindgenFn, BindgenStruct, Method};
use serde::*;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    *,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindgenItem {
    Fn(BindgenFn),
    Struct(BindgenStruct),
    Method(Method),
}

impl BindgenItem {
    pub fn raw_ident(&self) -> &str {
        match self {
            BindgenItem::Fn(item) => item.raw_ident(),
            BindgenItem::Struct(item) => item.raw_ident(),
            BindgenItem::Method(item) => item.raw_ident(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindgenItems(pub Vec<BindgenItem>);

impl Parse for BindgenItems {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = match input.call(Item::parse)? {
            Item::Fn(item) => vec![BindgenFn::from_signature(&item.sig).map(BindgenItem::Fn)?],
            Item::Struct(item) => vec![BindgenStruct::from_item(item).map(BindgenItem::Struct)?],
            Item::Impl(item) => parse_impl(item)?,

            item @ _ => {
                return Err(Error::new(
                    item.span(),
                    "`#[cs_bindgen]` is not supported on this type of item",
                ))
            }
        };

        Ok(BindgenItems(items))
    }
}

fn parse_impl(item: ItemImpl) -> syn::Result<Vec<BindgenItem>> {
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

    // Get the type name from the impl block.
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

    let ty = BindgenStruct::from_ident(ty_ident);

    // Example the impl item into a list of individual associated items. For now we
    // filter out everything except associated functions/methods.
    let methods = item
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(item) => Some(&item.sig),
            _ => None,
        })
        .map(|method| {
            let method = BindgenFn::from_signature(method)?;
            Ok(BindgenItem::Method(Method::new(ty.clone(), method)))
        })
        .collect::<syn::Result<_>>()?;

    Ok(methods)
}
