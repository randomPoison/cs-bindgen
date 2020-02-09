use serde::*;
use syn::ItemStruct;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindgenStruct {}

impl BindgenStruct {
    pub fn from_item(_item: ItemStruct) -> syn::Result<Self> {
        todo!()
    }
}
