use std::{fs, path::PathBuf};
use structopt::*;
use wasmtime::*;

fn main() {
    let opt = Opt::from_args();

    let store = Store::default();

    let test_wasm = fs::read(&opt.input).expect("Couldn't read mahjong.wasm");
    let module = Module::new(&store, &test_wasm).expect("Failed to create WASM module");
    let instance = Instance::new(&store, &module, &[]).expect("Failed to create module instance");

    for export in module.exports() {
        dbg!(export);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cs-bindgen")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}
