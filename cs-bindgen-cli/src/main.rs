use cs_bindgen_shared::*;
use std::{fs, path::PathBuf, str};
use structopt::*;
use wasmtime::*;

fn main() {
    let opt = Opt::from_args();

    let store = Store::default();

    let test_wasm = fs::read(&opt.input).expect("Couldn't read mahjong.wasm");
    let module = Module::new(&store, &test_wasm).expect("Failed to create WASM module");
    let instance = Instance::new(&store, &module, &[]).expect("Failed to create module instance");

    let len_fn = instance
        .find_export_by_name("__cs_bindgen_decl_len_generate_tileset_json")
        .expect("len fn not found")
        .func()
        .expect("len fn wasn't a fn???")
        .borrow();

    let decl_fn = instance
        .find_export_by_name("__cs_bindgen_decl_ptr_generate_tileset_json")
        .expect("decl fn not found")
        .func()
        .expect("decl fn wasn't a fn???")
        .borrow();

    let decl_ptr = decl_fn.call(&[]).expect("Failed to call decl fn")[0].unwrap_i32() as usize;
    let len = len_fn.call(&[]).expect("Failed to call len fn")[0].unwrap_i32() as usize;

    let memory = instance
        .find_export_by_name("memory")
        .expect("memory not found")
        .memory()
        .expect("memory wasn't a memory???")
        .borrow();

    // SAFETY: `Memory::data` is (maybe) only unsafe if using wasmtime from multiple
    // threads? That's at least what the safety note indicates, probably worth doing
    // further research.
    let memory_bytes = unsafe { memory.data() };

    let decl_bytes = &memory_bytes[decl_ptr..decl_ptr + len];

    let decl = str::from_utf8(decl_bytes).expect("decl not valid utf8");

    let bindgen_fn =
        serde_json::from_str::<BindgenFn>(&decl).expect("Failed to deserialize bindgen fn decl");
    dbg!(&bindgen_fn);
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cs-bindgen")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}
