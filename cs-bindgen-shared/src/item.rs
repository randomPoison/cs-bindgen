use crate::{BindgenFn, BindgenStruct};
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
}

impl Parse for BindgenItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item = match input.call(Item::parse)? {
            Item::Fn(item) => BindgenFn::from_item(item).map(BindgenItem::Fn)?,
            Item::Struct(item) => BindgenStruct::from_item(item).map(BindgenItem::Struct)?,

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
