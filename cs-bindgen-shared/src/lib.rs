use derive_more::From;
use schematic::Schema;
use schematic::Struct;
use serde::*;
use std::borrow::Cow;

// Re-export schematic so that dependent crates don't need to directly depend on it.
pub use schematic;

/// An item exported from the Rust as a language binding.
#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum Export {
    Fn(Func),
    Struct(Struct),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Func {
    pub name: Cow<'static, str>,
    pub binding: Cow<'static, str>,
    pub receiver: Option<Receiver>,
    pub inputs: Vec<(Option<Cow<'static, str>>, Schema)>,
    pub output: Schema,
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
