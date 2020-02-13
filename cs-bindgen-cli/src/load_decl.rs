use crate::Opt;
use cs_bindgen_shared::Export;
use snafu::*;
use std::{fs, io, path::PathBuf, str};
use wasmtime::*;

static DECL_PTR_FN_PREFIX: &str = "__cs_bindgen_decl_ptr_";

pub fn load_declarations(opt: &Opt) -> Result<Vec<Export>, Error> {
    let store = Store::default();

    let test_wasm = fs::read(&opt.input).context(LoadModule {
        path: opt.input.clone(),
    })?;
    let module = Module::new(&store, &test_wasm).context(InstantiateModule)?;
    let instance = Instance::new(&store, &module, &[]).context(InstantiateModule)?;

    let memory = instance
        .find_export_by_name("memory")
        .and_then(Extern::memory)
        .context(InvalidModule {
            message: "`memory` not found in module, or was not a `Memory` extern",
        })?
        .borrow();

    // Find any exported declarations and extract the declaration data from the module.
    let mut decls = Vec::new();
    for func in module.exports() {
        if func.name().starts_with("__cs_bindgen_decl_ptr_") {
            let fn_suffix = &func.name()[DECL_PTR_FN_PREFIX.len()..];

            // Get the decl function from the instance.
            let decl_fn = instance
                .find_export_by_name(func.name())
                .unwrap_or_else(|| {
                    panic!(
                        "Failed to find export `{}` declared in module exports. This likely \
                        indicates a bug in the `wasmtime` crate",
                        func.name()
                    )
                })
                .func()
                .context(DeclFunction { name: func.name() })?
                .borrow();

            // Get the length function from the instance.
            let len_fn_name = format!("__cs_bindgen_decl_len_{}", fn_suffix);
            let len_fn = instance
                .find_export_by_name(&len_fn_name)
                .and_then(Extern::func)
                .context(DeclFunction { name: &len_fn_name })?
                .borrow();

            // Invoke both to get the pointer to the decl string and the length of the string.
            let decl_ptr = decl_fn
                .call(&[])
                .context(Invoke { name: func.name() })
                .and_then(|ret| extract_return(ret, func.name()))?;
            let len = len_fn
                .call(&[])
                .context(Invoke { name: &len_fn_name })
                .and_then(|ret| extract_return(ret, &len_fn_name))?;

            let decl = deserialize_decl_string(&memory, decl_ptr, len, fn_suffix)?;

            decls.push(decl);
        }
    }

    Ok(decls)
}

fn extract_return<S: Into<String>>(ret: Box<[Val]>, name: S) -> Result<i32, Error> {
    ensure!(ret.len() == 1, BadReturn { name, ret });
    ret[0].i32().context(BadReturn { name, ret })
}

fn deserialize_decl_string(
    memory: &Memory,
    decl_ptr: i32,
    len: i32,
    name: &str,
) -> Result<Export, Error> {
    // Convert the pointer and len to `usize` so that we can index into the byte array.
    let decl_ptr = decl_ptr as usize;
    let len = len as usize;

    // SAFETY: `Memory::data` is safe as long as we don't do anything that would
    // invalidate the reference while we're borrowing the memory. Specifically:
    //
    // * Explicitly calling `Memory::grow` (duh).
    // * Invoking a function in the module that contains the `memory.grow` instruction.
    //
    // That second one is the more critical one, because it means we have to make sure
    // we don't invoke *any* function in the module while borrowing the memory. For
    // our purposes that's fine, and we can probably write a safe wrapper function that
    // copies out the specified data so that we don't have to hold the borrow on the
    // memory.
    let memory_bytes = unsafe { memory.data() };

    let decl_bytes = &memory_bytes[decl_ptr..decl_ptr + len];
    let decl_str = str::from_utf8(decl_bytes).context(BadDeclString { name })?;
    serde_json::from_str(&decl_str).context(BadDecl { name })
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not load file from path {}: {}", path.display(), source))]
    LoadModule { path: PathBuf, source: io::Error },

    #[snafu(display("Could not instantiate wasm module: {}", source))]
    InstantiateModule { source: anyhow::Error },

    #[snafu(display("WASM module was invalid: {}", message))]
    InvalidModule { message: &'static str },

    #[snafu(display("Exported item `{}` was missing or was not a function", name))]
    DeclFunction { name: String },

    #[snafu(display("Hit trap while invoking `{}`: {}", name, source))]
    Invoke { name: String, source: Trap },

    #[snafu(display("Declaration function `{}`: {:?}", name, ret))]
    BadReturn { name: String, ret: Box<[Val]> },

    #[snafu(display("Declaration string for `{}` was not valid utf-8: {}", name, source))]
    BadDeclString {
        name: String,
        source: str::Utf8Error,
    },

    #[snafu(display("Item declaration `{}` was invalid: {}", name, source))]
    BadDecl {
        name: String,
        source: serde_json::Error,
    },
}
