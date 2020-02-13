//! Example of what the code generated from `#[cs_bindgen]` should look like. Used
//! to verify that the generated code ABI is valid and will compile, and is useful
//! for understanding how the code generation works.

pub fn example_fn(first: u32, second: String) -> String {
    format!("first: {}, second: {}", first, second)
}

#[no_mangle]
pub unsafe extern "C" fn __cs_bindgen_generated__example_fn(
    first: <u32 as cs_bindgen::shared::abi::FromAbi>::Abi,
    second: <String as cs_bindgen::shared::abi::FromAbi>::Abi,
) -> <String as cs_bindgen::shared::abi::IntoAbi>::Abi {
    let first = cs_bindgen::shared::abi::FromAbi::from_abi(first);
    let second = cs_bindgen::shared::abi::FromAbi::from_abi(second);
    cs_bindgen::shared::abi::IntoAbi::into_abi(example_fn(first, second))
}
