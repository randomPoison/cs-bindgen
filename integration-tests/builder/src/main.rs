use regex::Regex;
use std::{env, fs, path::Path, process::Command};

fn main() {
    // Get the environment variables set by cargo so that we can put together the right
    // paths regardless of where in the directory structure this is invoked.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

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
        .arg(&bindings_path)
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

    // HACK: Manually insert some new lines into the generated code in order to improve
    // the formatter's output. For some inexplicable reason the C# formatter won't break
    // up long lines in all cases, which means that even after running the generated
    // code through the formatter it's still often unreadable. Notably:
    //
    // * If you have multiple statements on the same line, the formatter will leave them
    //   on the same line.
    // * It won't break attributes up onto multiple lines, and it won't put a new line
    //   between an attribute than the item the attribute is attached to, so all of our
    //   `[DllImport]` declarations are left on one big line even after formatting.
    //
    // Fortunately, if we manually insert new lines into those places, the formatter
    // keeps those new lines in place. So to force the formatter to break up the code
    // more, we manually insert '/n' after ';' (to break up statements) and ']' (to
    // break up attributes).
    let regex = Regex::new(r"(]|;)").unwrap();
    let file = fs::read_to_string(&bindings_path).unwrap();
    let result = regex.replace_all(&file, "$1\n");
    fs::write(&bindings_path, &*result).unwrap();
}
