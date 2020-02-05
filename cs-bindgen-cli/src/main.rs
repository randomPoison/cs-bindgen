use cs_bindgen_shared::*;
use heck::*;
use proc_macro2::TokenStream;
use quote::*;
use std::{ffi::OsStr, fs, fs::File, io::prelude::*, path::PathBuf, str};
use structopt::*;
use syn::{punctuated::Punctuated, token::Comma, *};
use wasmtime::*;

static DECL_PTR_FN_PREFIX: &str = "__cs_bindgen_decl_ptr_";

fn main() {
    let opt = Opt::from_args();

    let store = Store::default();

    let test_wasm = fs::read(&opt.input)
        .unwrap_or_else(|_| panic!("Failed to open wasm module: {}", opt.input.display()));
    let module = Module::new(&store, &test_wasm).expect("Failed to create WASM module");
    let instance = Instance::new(&store, &module, &[]).expect("Failed to create module instance");

    let memory = instance
        .find_export_by_name("memory")
        .expect("memory not found")
        .memory()
        .expect("memory wasn't a memory???")
        .borrow();

    // Find any exported declarations and extract the declaration data from the module.
    let mut decls = Vec::new();
    for func in module.exports() {
        if func.name().starts_with("__cs_bindgen_decl_ptr_") {
            let fn_suffix = &func.name()[DECL_PTR_FN_PREFIX.len()..];

            // Get the decl function from the instance.
            let decl_fn = instance
                .find_export_by_name(func.name())
                .expect("decl fn not found")
                .func()
                .expect("decl fn wasn't a fn???")
                .borrow();

            // Get the length function from the instance.
            let len_fn_name = format!("__cs_bindgen_decl_len_{}", fn_suffix);
            let len_fn = instance
                .find_export_by_name(&len_fn_name)
                .expect("len fn not found")
                .func()
                .expect("len fn wasn't a fn???")
                .borrow();

            // Invoke both to get the pointer to the decl string and the length of the string.
            let decl_ptr = decl_fn.call(&[]).expect("Failed to call decl fn")[0].unwrap_i32();
            let len = len_fn.call(&[]).expect("Failed to call len fn")[0].unwrap_i32();

            let decl = deserialize_decl_string(&memory, decl_ptr, len)
                .expect("Failed to deserialize decl string");

            decls.push(decl);
        }
    }

    // Generate the C# binding code.
    // ---------------------------------------------------------------------------------------------

    let dll_name = opt
        .input
        .file_stem()
        .and_then(OsStr::to_str)
        .expect("Unable to get name of wasm file");

    let class_name = format_ident!("{}", dll_name.to_camel_case());

    let fn_bindings = decls.iter().map(|decl| quote_bindgen_fn(decl, dll_name));

    let result = quote! {
        using System;
        using System.Runtime.InteropServices;
        using System.Text;

        public class #class_name
        {
            [DllImport(
                #dll_name,
                EntryPoint = "__cs_bindgen_drop_string",
                CallingConvention = CallingConvention.Cdecl)]
            private static extern void DropString(RustOwnedString raw);

            #( #fn_bindings )*

            [StructLayout(LayoutKind.Sequential)]
            private struct RustOwnedString
            {
                public IntPtr Ptr;
                public ulong Length;
                public ulong Capacity;
            }
        }
    }
    .to_string();

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

fn deserialize_decl_string(
    memory: &Memory,
    decl_ptr: i32,
    len: i32,
) -> serde_json::Result<BindgenFn> {
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
    let decl_str = str::from_utf8(decl_bytes).expect("decl not valid utf8");
    serde_json::from_str(&decl_str)
}

fn quote_bindgen_fn(bindgen_fn: &BindgenFn, dll_name: &str) -> TokenStream {
    let entry_point = bindgen_fn.generated_name();
    let raw_binding = format_ident!("__{}", bindgen_fn.raw_ident().to_camel_case());
    let binding_return_ty = match bindgen_fn.ret {
        None => quote! { void },
        Some(prim) => quote_primitive_binding(prim),
    };

    let binding_args = bindgen_fn
        .args
        .iter()
        .map(|arg| {
            let ident = arg.ident();
            let ty = quote_primitive_binding(arg.ty);
            quote! { #ty #ident }
        })
        .collect::<Punctuated<_, Comma>>();

    let wrapper_fn = quote_wrapper_fn(&bindgen_fn, &raw_binding);

    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
        private static extern #binding_return_ty #raw_binding(#binding_args);

        #wrapper_fn
    }
}

fn quote_primitive_binding(return_ty: Primitive) -> TokenStream {
    match return_ty {
        Primitive::String => quote! { RustOwnedString },
        Primitive::Char => quote! { uint },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::F32 => quote! { float },
        Primitive::F64 => quote! { double },
        Primitive::Bool => quote! { byte },
    }
}

fn quote_primitive(return_ty: Primitive) -> TokenStream {
    match return_ty {
        Primitive::String => quote! { string },
        Primitive::Char => quote! { uint },
        Primitive::I8 => quote! { sbyte },
        Primitive::I16 => quote! { short },
        Primitive::I32 => quote! { int },
        Primitive::I64 => quote! { long },
        Primitive::U8 => quote! { byte },
        Primitive::U16 => quote! { ushort },
        Primitive::U32 => quote! { uint },
        Primitive::U64 => quote! { ulong },
        Primitive::F32 => quote! { float },
        Primitive::F64 => quote! { double },
        Primitive::Bool => quote! { bool },
    }
}

fn quote_wrapper_fn(bindgen_fn: &BindgenFn, raw_binding: &Ident) -> TokenStream {
    let cs_fn_name = format_ident!("{}", bindgen_fn.raw_ident().to_camel_case());
    let cs_return_ty = match bindgen_fn.ret {
        None => quote! { void },
        Some(prim) => quote_primitive(prim),
    };

    // Build the list of arguments to the wrapper function.
    let args = bindgen_fn
        .args
        .iter()
        .map(|arg| {
            let ident = arg.ident();
            let ty = quote_primitive(arg.ty);
            quote! { #ty #ident }
        })
        .collect::<Punctuated<_, Comma>>();

    // Build the list of arguments to the wrapper function.
    let invoke_args = bindgen_fn
        .args
        .iter()
        .map(|arg| match arg.ty {
            Primitive::String => unimplemented!("Don't know how to pass a `string` to rust"),

            Primitive::Bool => {
                let ident = arg.ident();
                quote! { (#ident ? 1 : 0) }
            }

            _ => arg.ident().into_token_stream(),
        })
        .collect::<Punctuated<_, Comma>>();

    let invoke_expr = match &bindgen_fn.ret {
        None => quote! { #raw_binding(#invoke_args); },

        Some(prim) => {
            let invoke_expr = quote! { var rawResult = #raw_binding(#invoke_args); };

            let result_expr = match prim {
                Primitive::String => quote! {
                    string result;
                    unsafe
                    {
                        result = Encoding.UTF8.GetString((byte*)rawResult.Ptr, (int)rawResult.Length);
                    }

                    DropString(rawResult);

                    return result;
                },

                Primitive::Bool => quote! {
                    return rawResult != 0;
                },

                _ => quote! { return rawResult; },
            };

            quote! {
                #invoke_expr

                #result_expr
            }
        }
    };

    quote! {
        public static #cs_return_ty #cs_fn_name(#args)
        {
            // TODO: Process args so they're ready to pass to the rust fn.

            #invoke_expr
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cs-bindgen")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
}
