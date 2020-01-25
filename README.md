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
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

Call into Rust code from C# as if it were idiomatic C#!

```cs
var greeting = Example.Greet("Ferris");

// Prints "Hello, Ferris!"
Console.WriteLine(greeting);
```

> NOTE: Above example not yet fully supported. Notably, it's not yet possible to pass a string from C# to Rust (though returning one from Rust to C# work fine).

## Status

Highly experimental! Do not use, even as a joke!
