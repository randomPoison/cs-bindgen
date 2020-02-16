use derive_more::From;
use serde::*;
use std::borrow::Cow;

// Re-export schematic so that dependent crates don't need to directly depend on it.
pub use schematic;
pub use schematic::{Schema, Struct};

pub fn serialize_export<E: Into<Export>>(export: E) -> String {
    let export = export.into();
    serde_json::to_string(&export).expect("Failed to serialize export")
}

/// An item exported from the Rust as a language binding.
#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum Export {
    Fn(Func),
    Method(Method),
    Struct(Struct),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Func {
    pub name: Cow<'static, str>,
    pub binding: Cow<'static, str>,
    pub inputs: Vec<(Cow<'static, str>, Schema)>,
    pub output: Schema,
}

impl Func {
    pub fn inputs(&self) -> impl Iterator<Item = (&str, &Schema)> {
        self.inputs.iter().map(|(name, schema)| (&**name, schema))
    }
}

#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub struct Method {
    pub name: Cow<'static, str>,
    pub binding: Cow<'static, str>,
    pub self_type: Schema,
    pub receiver_style: ReceiverStyle,
    pub inputs: Vec<(Cow<'static, str>, Schema)>,
    pub output: Schema,
}

impl Method {
    pub fn inputs(&self) -> impl Iterator<Item = (&str, &Schema)> {
        self.inputs.iter().map(|(name, schema)| (&**name, schema))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Receiver {
    pub ty: Schema,
    pub style: ReceiverStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReceiverStyle {
    Move,
    Ref,
    RefMut,
}
