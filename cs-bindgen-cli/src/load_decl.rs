use crate::Opt;
use cs_bindgen_shared::Export;
use parity_wasm::elements::ExportEntry;
use snafu::*;
use std::{fs, io, path::PathBuf, str};
use wasmi::{ImportsBuilder, Module, ModuleInstance, NopExternals, RuntimeValue};

static DECL_PTR_FN_PREFIX: &str = "__cs_bindgen_decl_ptr_";

pub fn load_declarations(opt: &Opt) -> Result<Vec<Export>, Error> {
    // Load the WASM module from the specified file.
    let module = parity_wasm::deserialize_file(&opt.input).context(LoadModule)?;

    let decl_exports = module
        .export_section()
        .context(InvalidModule {
            message: "Wasm module has no exports",
        })?
        .entries()
        .iter()
        .map(ExportEntry::field)
        .filter(|name| name.starts_with(DECL_PTR_FN_PREFIX))
        .map(Into::into)
        .collect::<Vec<String>>();

    // Instantiate a module with empty imports and
    // assert that there is no `start` function.
    let module = Module::from_parity_wasm_module(module)?;
    let instance =
        ModuleInstance::new(&module, &ImportsBuilder::default())?.run_start(&mut NopExternals)?;

    // Find any exported declarations and extract the declaration data from the module.
    let mut decls = Vec::new();
    for func in decl_exports {
        let result_string_addr = instance
            .invoke_export(&func, &[], &mut NopExternals)
            .context(Invoke { name: func })?
            .context();
        dbg!(&result_string_addr);

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
    #[snafu(display("Could not load Wasm module: {}", source))]
    LoadModule {
        source: parity_wasm::elements::Error,
    },

    #[snafu(display("WASM module was invalid: {}", message))]
    InvalidModule { message: &'static str },

    #[snafu(
        display("An error occurred instantiating the Wasm module: {}", source),
        context(false)
    )]
    WasmError { source: wasmi::Error },

    #[snafu(
        display("An error occurred while starting Wasm module: {}", source),
        context(false)
    )]
    StartError { source: wasmi::Trap },

    #[snafu(display("Error while invoking `{}`: {}", name, source))]
    Invoke { name: String, source: wasmi::Error },

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
