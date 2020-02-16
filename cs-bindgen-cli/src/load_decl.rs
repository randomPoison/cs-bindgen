use crate::Opt;
use cs_bindgen_shared::Export;
use failure::Error;
use parity_wasm::elements::ExportEntry;
use std::str;
use wasmi::{ExternVal, ImportsBuilder, Module, ModuleInstance, NopExternals};

static DECL_PTR_FN_PREFIX: &str = "__cs_bindgen_describe__";

/// Loads the specified Wasm module and extracts the export declarations.
pub fn load_declarations(opt: &Opt) -> Result<Vec<Export>, Error> {
    // Load the WASM module from the specified file.
    let module = parity_wasm::deserialize_file(&opt.input)?;

    let descriptor_fns = module
        .export_section()
        .ok_or(failure::err_msg("No exports found in Wasm module"))?
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

    let memory = instance.export_by_name("memory");
    let memory = memory
        .as_ref()
        .and_then(ExternVal::as_memory)
        .ok_or(failure::err_msg("No memory export found in Wasm module"))?;

    // Find any exported declarations and extract the declaration data from the module.
    let mut exports = Vec::new();
    for func in descriptor_fns {
        let result_string_addr = instance
            .invoke_export(&func, &[], &mut NopExternals)?
            .ok_or(failure::err_msg("Decl function didn't return a value"))?
            .try_into::<i32>()
            .ok_or(failure::err_msg("Decl function didn't return an `i32`"))?;

        // Get the bytes of the `RawVec<u8>` struct that was created.
        let str_ptr = memory.get_value::<u32>(result_string_addr as u32)?;
        let str_len = memory.get_value::<u32>(result_string_addr as u32 + 4)?;

        // Get the JSON string returned by the descriptor function.
        let json_bytes = memory.get(str_ptr, str_len as usize)?;
        let json = str::from_utf8(&json_bytes)?;

        // Deserialize the export and add it to the list.
        let export = serde_json::from_str(json)?;
        exports.push(export);
    }

    Ok(exports)
}
