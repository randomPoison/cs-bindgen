use crate::{BindgenFn, BindgenStruct};
use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindgenItem {
    Fn(BindgenFn),
    Struct(BindgenStruct),
}
