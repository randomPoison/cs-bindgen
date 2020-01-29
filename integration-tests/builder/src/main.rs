use std::{env, path::Path, process::Command};

fn main() {
    // Get the environment variables set by cargo so that we can put together the right
    // paths regardless of where in the directory structure this is invoked.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    dbg!(&manifest_dir);

    let wasm_module_path =
        manifest_dir.join("../../target/wasm32-unknown-unknown/debug/integration_tests.wasm");
    let bindings_path = manifest_dir.join("../TestRunner/Bindings.cs");

    // Build the WASM module for the test project so that we can run it through the
    // cs-bindgen CLI tool.
    println!("Building WASM module for integration-tests:");

    let mut child = Command::new("cargo")
        .args(&[
            "build",
            "--target=wasm32-unknown-unknown",
            "-p=integration-tests",
        ])
        .spawn()
        .expect("Failed to spawn the build process");

    let status = child.wait().expect("Failed to finish build process");
    if !status.success() {
        panic!("Build process exited with an error");
    }

    // Run the code generation script.
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("-p=cs-bindgen-cli")
        .arg("--")
        .arg(wasm_module_path)
        .arg("-o")
        .arg(bindings_path)
        .spawn()
        .expect("Failed to spawn cs-bindgen process");

    let status = child.wait().expect("Failed to finish codegen process");
    if !status.success() {
        panic!("Codegen process finished with an error");
    }

    // Build the actual DLL for the project.
    let mut child = Command::new("cargo")
        .args(&["build", "-p=integration-tests"])
        .spawn()
        .expect("Failed to spawn the build process");

    let status = child.wait().expect("Failed to finish building the dylib");
    if !status.success() {
        panic!("Dylib build process finished with an error");
    }
}
