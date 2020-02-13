use serde::*;

/// Metadata for an exported function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Func {
    /// The identifier for the original function.
    pub ident: String,

    /// The identifier for the generated binding function.
    pub binding: String,

    pub receiver: Option<Receiver>,

    /// List of argument names.
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Receiver {
    Ref,
    RefMut,
    Value,
}
