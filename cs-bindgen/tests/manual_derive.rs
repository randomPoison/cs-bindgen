//! Example of what the code generated from `#[cs_bindgen]` should look like. Used
//! to verify that the generated code ABI is valid and will compile, and is useful
//! for understanding how the code generation works.

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
    use cs_bindgen::shared::{schematic::encode, Func};

    let schema = Func {
        name: "example_fn".into(),
        binding: "__cs_bindgen_generated__example_fn".into(),
        receiver: None,
        inputs: vec![
            (
                Some("first".into()),
                encode::<u32>().expect("Failed to generate schema for argument"),
            ),
            (
                Some("second".into()),
                encode::<String>().expect("Failed to generate schema for argument"),
            ),
        ],
        output: encode::<String>().expect("Failed to generate schema for return type"),
    };

    Box::new(
        cs_bindgen::serde_json::to_string(&schema)
            .expect("Failed to serialize schema")
            .into(),
    )
}
