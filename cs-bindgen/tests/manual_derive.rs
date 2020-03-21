//! Example of what the code generated from `#[cs_bindgen]` should look like. Used
//! to verify that the generated code ABI is valid and will compile, and is useful
//! for understanding how the code generation works.

use cs_bindgen::{abi::*, shared::*};
use schematic::*;

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
    first: <u32 as Abi>::Abi,
    second: <String as Abi>::Abi,
) -> <String as Abi>::Abi {
    let first = Abi::from_abi(first);
    let second = Abi::from_abi(second);
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

impl Abi for ExampleStruct {
    type Abi = *mut Self;

    fn into_abi(self) -> Self::Abi {
        std::boxed::Box::into_raw(std::boxed::Box::new(self))
    }

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        *std::boxed::Box::from_raw(abi)
    }
}

impl<'a> Abi for &'a ExampleStruct {
    type Abi = Self;

    fn into_abi(self) -> Self::Abi {
        self
    }

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi
    }
}

impl<'a> Abi for &'a mut ExampleStruct {
    type Abi = *mut ExampleStruct;

    fn into_abi(self) -> Self::Abi {
        self as *mut _
    }

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

impl Abi for SimpleEnum {
    type Abi = isize;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        match abi {
            0 => Self::Foo,
            1 => Self::Bar,
            2 => Self::Baz,

            _ => panic!("Unknown discriminant {} for `SimpleEnum`", abi),
        }
    }

    fn into_abi(self) -> Self::Abi {
        self as _
    }
}

pub enum ComplexEnum {
    Foo,
    Bar(String, u32),
    Baz { first: SimpleEnum, second: String },
}

impl Abi for ComplexEnum {
    type Abi = RawEnum<isize, ComplexEnum_Abi>;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        match abi.discriminant {
            0 => Self::Foo,

            1 => {
                let value = abi.value.assume_init().Bar;
                Self::Bar(
                    Abi::from_abi(value.element_0),
                    Abi::from_abi(value.element_1),
                )
            }

            2 => {
                let value = abi.value.assume_init().Baz;
                Self::Baz {
                    first: Abi::from_abi(value.first),
                    second: Abi::from_abi(value.second),
                }
            }

            _ => panic!(
                "Unknown discriminant {} for `ComplexEnum`",
                abi.discriminant
            ),
        }
    }

    fn into_abi(self) -> Self::Abi {
        match self {
            Self::Foo => RawEnum::unit(0),

            Self::Bar(element_0, element_1) => RawEnum::new(
                1,
                ComplexEnum_Abi {
                    Bar: ComplexEnum_Abi_Bar {
                        element_0: element_0.into_abi(),
                        element_1: element_1.into_abi(),
                    },
                },
            ),

            Self::Baz { first, second } => RawEnum::new(
                2,
                ComplexEnum_Abi {
                    Baz: ComplexEnum_Abi_Baz {
                        first: first.into_abi(),
                        second: second.into_abi(),
                    },
                },
            ),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(bad_style)]
pub union ComplexEnum_Abi {
    Bar: ComplexEnum_Abi_Bar,
    Baz: ComplexEnum_Abi_Baz,
}

unsafe impl AbiPrimitive for ComplexEnum_Abi {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComplexEnum_Abi_Bar {
    element_0: <String as Abi>::Abi,
    element_1: <u32 as Abi>::Abi,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComplexEnum_Abi_Baz {
    first: <SimpleEnum as Abi>::Abi,
    second: <String as Abi>::Abi,
}
