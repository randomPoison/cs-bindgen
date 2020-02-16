<div align="center">

  <h1><code>schematic</code></h1>

  <p>
    <strong>Generate serde-compatible type schemas for Rust.</strong>
  </p>

</div>

## Example

```rust
#[derive(Serialize, Deserialize, Schema)]
pub struct MyStruct {
    name: String,
    value: u32,
}

let schema = schematic::encode::<MyStruct>();
println!("{:?}", schema);
```

## Status

Highly experimental! Do not use, but please contribute! :heart:

## Why `schematic`?

Rust has excellent support for serializing and deserializing data via the popular [Serde] crate. However, in many situations it's not enough to just be able to serialize to-and-from a data format: It's important to be able to also reason about the structure of that serialized data. While serde does an excellent job handling serialization for *values*, it [doesn't support describing types](https://github.com/serde-rs/serde/issues/345).

It's often useful to be able to describe the format of serialized data in a language-independent way in order to facilitate language interop or use with external tooling. Here are some examples of how Schematic can help in these cases:

* For web developers looking to make use of [OpenAPI] and [Swagger], it's necessary to be able to describe the format data returned from (or accepted by) web APIs.
* In order to generate high-level language bindings to Rust for other languages, it's necessary to be able to export descriptions of Rust types. The [cs-bindgen] tooling uses `schematic` in order to generate C# types that match exported Rust types, providing a smooth experience integrating a Rust crate into a C# project.
* For game development, it's useful to have a graphical editor for game assets. Such an editor will usually be responsible for generating and modifying data for assets in a serialized format. Such an editor will need to know the expected format of asset data in order to generate data in a format that can be deserialized at runtime.

## Schematic and Serde

Schematic is specifically designed to support describing types in a way that accurately describes how the data will be serialized when using Serde.

> TODO: Compare the Schematic data model with the Serde data model.

## Limitations

Schematic does not support fully describing the Rust type system. Specifically, Schematic cannot describe the following type constructs:

* Unions
* Functions
* Closures
* Function pointers
* Trait objects
* `impl Trait` expressions

Schematic only attempts to describe data, focussing primarily on describing data types that can be serialized by Serde.

[Serde]: https://serde.rs/
[OpenAPI]: https://swagger.io/resources/open-api/
[Swagger]: https://swagger.io/
[cs-bindgen]: https://github.com/randomPoison/cs-bindgen
