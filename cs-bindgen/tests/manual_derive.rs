//! Example of what the code generated from `#[cs_bindgen]` should look like. Used
//! to verify that the generated code ABI is valid and will compile, and is useful
//! for understanding how the code generation works.
//!
//! For an exported function, we generate two items:
//!
//! * The binding function, which is exported as `extern "C"` and handles the
//!   boilerplate work of converting two and from ABI-compatible types.
//! * The describe function, which

pub fn example_fn(first: u32, second: String) -> String {
    format!("first: {}, second: {}", first, second)
}

#[no_mangle]
pub unsafe extern "C" fn __cs_bindgen_generated__example_fn(
    first: <u32 as cs_bindgen::abi::FromAbi>::Abi,
    second: <String as cs_bindgen::abi::FromAbi>::Abi,
) -> <String as cs_bindgen::abi::IntoAbi>::Abi {
    let first = cs_bindgen::abi::FromAbi::from_abi(first);
    let second = cs_bindgen::abi::FromAbi::from_abi(second);
    cs_bindgen::abi::IntoAbi::into_abi(example_fn(first, second))
}

#[no_mangle]
pub unsafe extern "C" fn __cs_bindgen_describe__example_fn() -> Box<cs_bindgen::abi::RawVec<u8>> {
    use cs_bindgen::shared::{schematic::describe, Func};

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

    Box::new(cs_bindgen::shared::serialize_export(export).into())
}

pub struct ExampleStruct {
    _field: String,
}

impl cs_bindgen::shared::schematic::Describe for ExampleStruct {
    fn describe<E>(describer: E) -> Result<E::Ok, E::Error>
    where
        E: cs_bindgen::shared::schematic::Describer,
    {
        let describer =
            describer.describe_struct(cs_bindgen::shared::schematic::type_name!(ExampleStruct))?;
        cs_bindgen::shared::schematic::DescribeStruct::end(describer)
    }
}

impl cs_bindgen::abi::IntoAbi for ExampleStruct {
    type Abi = *mut Self;

    fn into_abi(self) -> Self::Abi {
        std::boxed::Box::into_raw(std::boxed::Box::new(self))
    }
}

impl cs_bindgen::abi::FromAbi for ExampleStruct {
    type Abi = *mut Self;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        *std::boxed::Box::from_raw(abi)
    }
}

impl<'a> cs_bindgen::abi::IntoAbi for &'a ExampleStruct {
    type Abi = Self;

    fn into_abi(self) -> Self::Abi {
        self
    }
}

impl<'a> cs_bindgen::abi::FromAbi for &'a ExampleStruct {
    type Abi = Self;

    unsafe fn from_abi(abi: Self::Abi) -> Self {
        abi
    }
}
