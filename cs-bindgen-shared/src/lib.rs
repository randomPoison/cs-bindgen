use schematic::Schema;
use serde::*;

pub mod meta;

// Re-export schematic so that dependent crates don't need to directly depend on it.
pub use schematic;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Describe {
    pub name: String,
    pub receiver: Option<Receiver>,
    pub inputs: Vec<Schema>,
    pub output: Schema,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Receiver {
    pub name: String,
    pub style: ReceiverStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReceiverStyle {
    Move,
    Ref,
    RefMut,
}
