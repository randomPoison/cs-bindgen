use derive_more::From;
use serde::*;
use std::borrow::Cow;

// Re-export schematic so that dependent crates don't need to directly depend on it.
pub use schematic;
pub use schematic::{Schema, TypeName};

pub fn serialize_export<E: Into<Export>>(export: E) -> String {
    let export = export.into();
    serde_json::to_string(&export).expect("Failed to serialize export")
}

/// An item exported from the Rust as a language binding.
#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum Export {
    Fn(Func),
    Method(Method),
    Named(NamedType),
}

/// A free function exported from the Rust lib.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Func {
    /// The original name of the function, as declared in the Rust source code.
    ///
    /// The function with this name cannot be called directly in the built lib. The
    /// value of `binding` specifies the name of the generated binding function.
    pub name: Cow<'static, str>,

    /// The name of the generated binding function.
    ///
    /// This is the exported function that is directly accessible in the generated
    /// lib. This name isn't meant to be user-facing, and should only be used
    /// internally by the generated language bindings to call into the lib. For the
    /// "true" name of the function, see `name`.
    pub binding: Cow<'static, str>,

    /// The argument types for the function.
    ///
    /// Note that these are the types of the original function, NOT the generated
    /// binding function.
    pub inputs: Vec<FnArg>,

    /// The return type of the function.
    ///
    /// Note that this is the return type of the original function, NOT the generated
    /// binding function.
    pub output: Option<Repr>,
}

/// A user-defined type (i.e. a struct or an enum).
///
/// Both structs and enums are exported as "named types", since there are a number
/// of configuration options that are shared for all exported types. To determine
/// the full details of the exported type, examine the include `schema`.
///
/// An exported name type can only be a struct or an enum, as exporting unions is
/// not supported.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamedType {
    pub type_name: TypeName,
    pub binding_style: BindingStyle,

    pub index_fn: Cow<'static, str>,
    pub drop_vec_fn: Cow<'static, str>,
    pub convert_list_fn: Cow<'static, str>,
}

impl NamedType {
    pub fn schema(&self) -> Option<&Schema> {
        match &self.binding_style {
            BindingStyle::Value(schema) => Some(schema),
            BindingStyle::Handle => None,
        }
    }
}

#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub struct Method {
    pub name: Cow<'static, str>,
    pub binding: Cow<'static, str>,
    pub self_type: TypeName,
    pub receiver: Option<ReceiverStyle>,
    pub inputs: Vec<FnArg>,
    pub output: Option<Repr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnArg {
    pub name: Cow<'static, str>,
    pub repr: Repr,
}

impl FnArg {
    pub fn new<N>(name: N, repr: Repr) -> Self
    where
        N: Into<Cow<'static, str>>,
    {
        Self {
            name: name.into(),
            repr,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReceiverStyle {
    Move,
    Ref,
    RefMut,
}

/// The style of binding generated for an exported type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BindingStyle {
    /// The type is exported as a class wrapping an opaque handle.
    Handle,

    /// Values of the type are marshalled directly into C# values.
    Value(Schema),
}

/// The supported type representations that can be passed across the FFI boundary.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Repr {
    Unit,
    Bool,
    Char,

    I8,
    I16,
    I32,
    I64,
    ISize,

    U8,
    U16,
    U32,
    U64,
    USize,

    F32,
    F64,

    /// A user defined type.
    ///
    /// The referenced type must be included in the set of exported types, such that the
    /// given `TypeName` can be used to look up the full schema and metadata for the
    /// type.
    Named(TypeName),

    /// An owned pointer.
    Box(Box<Repr>),

    /// A borrowed pointer.
    Ref(Box<Repr>),

    /// An owned array of elements.
    Vec(Box<Repr>),

    /// A borrowed array of elements.
    Slice(Box<Repr>),

    /// An array of elements
    Array {
        element: Box<Repr>,
        len: usize,
    },

    /// An owned string.
    String,

    /// A borrowed string slice.
    Str,

    /// An optional value.
    Option(Box<Repr>),

    /// The result of a fallible operation.
    Result {
        ok: Box<Repr>,
        err: Box<Repr>,
    },
}

impl Repr {
    /// Gets the repr for some user-defined type `T`.
    ///
    /// Returns [`Repr::Named`] with the [`TypeName`] returned by `T`'s
    /// [`Named::type_name`] impl.
    pub fn named<T: Named>() -> Self {
        Repr::Named(T::type_name())
    }
}

/// A user-defined type.
pub trait Named {
    // TODO: Make this an associated constant once we stop using schematic and our
    // custom `TypeName` type doesn't require allocation.
    fn type_name() -> TypeName;
}
