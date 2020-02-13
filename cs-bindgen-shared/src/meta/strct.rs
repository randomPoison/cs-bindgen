use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
    pub ident: String,
}
