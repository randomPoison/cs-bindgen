<div align="center">

  <h1><code>cs-bindgen</code></h1>

  <p>
    <strong>Facilitating high-level interactions between Rust and C#.</strong>
  </p>

  <sub>Built with 🦀🔪 by <a href="https://randompoison.github.io/">a disgruntled Unity developer</a></sub>

</div>

## Example

Declare functions in Rust and expose them to C#.

```rust
#[cs_bindgen]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

Call into Rust code from C# as if it were idiomatic C#!

```cs
var greeting = Example.Greet("Ferris");

// Prints "Hello, Ferris!"
Console.WriteLine(greeting);
```

## Status

Highly experimental! Do not use, even as a joke!

## Setup

* Make sure you have the latest version of Rust installed:
  
  ```
  rustup update
  ````
* Install the `wasm32-unknown-unknown` toolchain:
  
  ```
  rustup target add wasm32-unknown-unknown
  ```
* Make sure you have the [.NET Core CLI installed](https://dotnet.microsoft.com/download) if you're going to run the integration test suite.

## Running Integration Tests

In addition to the usual Rust testing setup that can be run via `cargo run`, there's a more complete integration test setup that builds C# bindings into a .NET Core project and uses [xUnit](https://xunit.net/) to test that the Rust binary can be embedded correctly. To setup the bindings for the tests, first run:

```
cargo run -p builder
```

Then, to run the tests, navigate to the `integration-tests/TestRunner` directory and run:

```
dotnet test
```
