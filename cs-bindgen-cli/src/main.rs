use crate::load_decl::load_declarations;
use std::{fs, fs::File, io::prelude::*, path::PathBuf, process};
use structopt::*;

mod generate;
mod load_decl;

fn main() {
    let opt = Opt::from_args();
    let decls = match load_declarations(&opt) {
        Ok(decls) => decls,
        Err(err) => {
            // TODO: Provide suggestions for what users can do to resolve the issue.
            eprintln!("{}", err);
            process::abort();
        }
    };

    let result = generate::generate_bindings(decls, &opt);

    match opt.output {
        // If no output file was specified, print to stdout.
        None => println!("{}", result),

        // Write the generated code the specified output file.
        Some(out_path) => {
            // Make sure the output directory exists.
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create directory for output");
            }

            let mut file = File::create(&out_path).expect("Failed to open output file");
            file.write_all(result.as_bytes())
                .expect("Failed to write to output file");
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cs-bindgen")]
pub struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}
