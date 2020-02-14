//! Metadata for exported items captured from the Rust AST.
//!
//! The data in this module is intended to compliment the [type description
//! functionality][describe] by providing the non-type data that we can gather from
//! parsing the Rust source code. For example, item identifiers, function argument
//! names, attributes, and doc comment data aren't present in the [`Describe`]
//! output but are necessary in order to generate robust bindings.
//!
//! [describe]: ../describe/index.html
//! [`Describe`]: ../describe/trait.Describe.html

mod export;
mod func;
mod method;
mod strct;

pub use self::{
    export::Export,
    func::{Func, Receiver},
    method::Method,
    strct::Struct,
};
