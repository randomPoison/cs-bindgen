use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindgenImpl {
    pub ty_ident: String,
}

impl BindgenImpl {
    pub fn from_item(_item: syn::ItemImpl) -> syn::Result<Self> {
        todo!()
    }
}
