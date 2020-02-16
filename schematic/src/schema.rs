use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// In-memory representation of a type tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Schema {
    Struct(Box<Struct>),
    UnitStruct(UnitStruct),
    NewtypeStruct(Box<NewtypeStruct>),
    TupleStruct(TupleStruct),
    Enum(Box<Enum>),
    Option(Box<Schema>),
    Seq(Box<Schema>),
    Tuple(Vec<Schema>),
    Map {
        key: Box<Schema>,
        value: Box<Schema>,
    },
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    String,
    Unit,
}

impl Schema {
    pub fn as_struct(&self) -> Option<&Struct> {
        match self {
            Schema::Struct(inner) => Some(&**inner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitStruct {
    pub name: TypeName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NewtypeStruct {
    pub name: TypeName,
    pub inner: Schema,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Struct {
    pub name: TypeName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Enum {
    pub name: TypeName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TupleStruct {
    pub name: TypeName,
}

/// Unique name for a type.
///
/// All types are uniquely identified by a combination of their name and the module
/// in which they were declared; Since two types with the same name cannot be
/// declared in the same module, `TypeName` is always sufficient to disambiguate
/// between two types with the same name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeName {
    /// The local name of the type.
    pub name: Cow<'static, str>,

    /// The path to the module where the type is declared, starting with the crate name.
    ///
    /// Note that this may not be the same module that the type is publicly exported
    /// from in the owning crate.
    pub module: Cow<'static, str>,
}

/// Expands to a [`TypeName`] for the specified type.
///
/// When invoking this macro, a couple of things should be kept in mind in order to
/// get the correct result:
///
/// * This macro should be invoked in the same module that declares `$ty`, otherwise
///   the module path will not be correct.
/// * The given name should be unqualified, e.g. instead of `type_name!(foo::bar::Baz)`,
///   you should invoke it as `type_name!(Baz)`. This restriction may be lifted in
///   the future.
///
/// [`TypeName`]: struct.TypeName.html
#[macro_export]
macro_rules! type_name {
    ($ty:ty) => {
        $crate::TypeName {
            // TODO: Support stripping off
            name: std::borrow::Cow::Borrowed(stringify!($ty)),
            module: std::borrow::Cow::Borrowed(module_path!()),
        }
    };
}
