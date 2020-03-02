//! Example of what the code generated from `#[cs_bindgen]` should look like. Used
//! to verify that the generated code ABI is valid and will compile, and is useful
//! for understanding how the code generation works.

use cs_bindgen::{abi::*, shared::*};
use schematic::*;
use std::mem::ManuallyDrop;

// For an exported function, we generate two items:
//
// * The binding function, which is exported as `extern "C"` and handles the
//   boilerplate work of converting two and from ABI-compatible types.
// * The describe function, which

pub fn example_fn(first: u32, second: String) -> String {
    format!("first: {}, second: {}", first, second)
}

#[no_mangle]
pub unsafe extern "C" fn __cs_bindgen_generated__example_fn(
    first: <u32 as FromAbi>::Abi,
    second: <String as FromAbi>::Abi,
) -> <String as IntoAbi>::Abi {
    let first = FromAbi::from_abi(first);
    let second = FromAbi::from_abi(second);
    example_fn(first, second).into_abi()
}

#[no_mangle]
pub unsafe extern "C" fn __cs_bindgen_describe__example_fn() -> Box<RawVec<u8>> {
    let export = Func {
        name: "example_fn".into(),
        binding: "__cs_bindgen_generated__example_fn".into(),
        inputs: vec![
            (
                "first".into(),
                describe::<u32>().expect("Failed to generate schema for argument"),
            ),
            (
                "second".into(),
                describe::<String>().expect("Failed to generate schema for argument"),
            ),
        ],
        output: describe::<String>().expect("Failed to generate schema for return type"),
    };

    Box::new(serialize_export(export).into())
}

// When exporting a struct as a handle type (i.e. a class in C#) the ABI conversion
// simply boxes the value and then returns the pointer as an opaque handle.
// Additional `From/IntoAbi` impls are generated for references to the type in order
// to support passing/returning by reference.

pub struct ExampleStruct {
    pub field: String,
}

impl Describe for ExampleStruct {
    fn describe<E>(describer: E) -> Result<E::Ok, E::Error>
    where
        E: Describer,
    {
        let describer = describer.describe_struct(type_name!(ExampleStruct))?;
        describer.end()
    }
}

impl IntoAbi for ExampleStruct {
    type Abi = *mut Self;

    fn into_abi(self) -> Self::Abi {
        std::boxed::Box::into_raw(std::boxed::Box::new(self))
    }
}

impl FromAbi for ExampleStruct {
    type Abi = *mut Self;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        *std::boxed::Box::from_raw(abi)
    }
}

impl<'a> IntoAbi for &'a ExampleStruct {
    type Abi = Self;

    fn into_abi(self) -> Self::Abi {
        self
    }
}

impl<'a> FromAbi for &'a ExampleStruct {
    type Abi = Self;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi
    }
}

impl<'a> IntoAbi for &'a mut ExampleStruct {
    type Abi = *mut ExampleStruct;

    fn into_abi(self) -> Self::Abi {
        self as *mut _
    }
}

impl<'a> FromAbi for &'a mut ExampleStruct {
    type Abi = *mut ExampleStruct;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        &mut *abi
    }
}

// For enums, we have two potential ways to handle them:
//
// * C-like enums are are simply converted to an integer value.
// * Data-carrying enums are passed by value, which requires generating an
//   FFI-compatible representation that values of the enum can be converted
//   from/into.
//
// The raw representation of the enum is a integer discriminant paired with a union
// of all the fields. The `RawEnum` helper struct pairs the two together.

pub enum SimpleEnum {
    Foo,
    Bar,
    Baz,
}

impl FromAbi for SimpleEnum {
    type Abi = isize;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        match abi {
            0 => Self::Foo,
            1 => Self::Bar,
            2 => Self::Baz,

            _ => panic!("Unknown discriminant {} for `SimpleEnum`", abi),
        }
    }
}

impl IntoAbi for SimpleEnum {
    type Abi = isize;

    fn into_abi(self) -> Self::Abi {
        self as _
    }
}

pub enum ComplexEnum {
    Foo,
    Bar(String, u32),
    Baz { first: SimpleEnum, second: String },
}

impl FromAbi for ComplexEnum {
    type Abi = RawEnum<isize, ComplexEnumAbi>;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        match abi.discriminant {
            0 => Self::Foo,
            1 => {
                let value = ManuallyDrop::into_inner(abi.value.assume_init().Bar);
                Self::Bar(
                    FromAbi::from_abi(value.element_0),
                    FromAbi::from_abi(value.element_1),
                )
            }
            2 => {
                let value = ManuallyDrop::into_inner(abi.value.assume_init().Baz);
                Self::Baz {
                    first: FromAbi::from_abi(value.first),
                    second: FromAbi::from_abi(value.second),
                }
            }

            _ => panic!(
                "Unknown discriminant {} for `ComplexEnum`",
                abi.discriminant
            ),
        }
    }
}

impl IntoAbi for ComplexEnum {
    type Abi = RawEnum<isize, ComplexEnumAbi>;

    fn into_abi(self) -> Self::Abi {
        todo!();
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(bad_style)]
pub union ComplexEnumAbi {
    Bar: ManuallyDrop<ComplexEnumAbi_Bar>,
    Baz: ManuallyDrop<ComplexEnumAbi_Baz>,
}

unsafe impl AbiPrimitive for ComplexEnumAbi {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComplexEnumAbi_Bar {
    element_0: <String as FromAbi>::Abi,
    element_1: <u32 as FromAbi>::Abi,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComplexEnumAbi_Baz {
    first: <SimpleEnum as FromAbi>::Abi,
    second: <String as FromAbi>::Abi,
}
