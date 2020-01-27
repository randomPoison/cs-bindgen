use std::process::Command;

fn main() {
    // Build the WASM module for the test project so that we can run it through the
    // cs-bindgen CLI tool.
    println!("Building WASM module for integration-tests:");

    let mut child = Command::new("cargo")
        .args(&["build", "--target=wasm32-unknown-unknown"])
        .spawn()
        .expect("Failed to spawn the build process");

    let status = child.wait().expect("Failed to finish build process");
    if !status.success() {
        return;
    }

    // Run the code generation script.

    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "-p=cs-bindgen-cli",
            "../target/wasm32-unknown-unknown/debug/integration-tests.wasm",
            "-o=TestRunner/Bindings.cs",
        ])
        .spawn()
        .expect("Failed to spawn cs-bindgen process");

    let status = child.wait().expect("Failed to finish codegen process");
    if !status.success() {
        return;
    }
}
