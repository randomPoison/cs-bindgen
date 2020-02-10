use crate::{BindgenFn, BindgenImpl, BindgenStruct};
use serde::*;
use std::borrow::Cow;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    *,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindgenItem {
    Fn(BindgenFn),
    Struct(BindgenStruct),
    Impl(BindgenImpl),
}

impl BindgenItem {
    pub fn raw_ident(&self) -> Cow<str> {
        match self {
            BindgenItem::Fn(item) => item.raw_ident().into(),
            BindgenItem::Struct(item) => item.raw_ident().into(),
            BindgenItem::Impl(item) => format!("impl__{}", item.ty_ident).into(),
        }
    }
}

impl Parse for BindgenItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item = match input.call(Item::parse)? {
            Item::Fn(item) => BindgenFn::from_item(item).map(BindgenItem::Fn)?,
            Item::Struct(item) => BindgenStruct::from_item(item).map(BindgenItem::Struct)?,
            Item::Impl(item) => BindgenImpl::from_item(item).map(BindgenItem::Impl)?,

            item @ _ => {
                return Err(Error::new(
                    item.span(),
                    "`#[cs_bindgen]` is not supported on this type of item",
                ))
            }
        };

        Ok(item)
    }
}
