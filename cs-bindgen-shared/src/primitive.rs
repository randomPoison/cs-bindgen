use proc_macro2::TokenStream;
use quote::*;
use serde::*;
use syn::*;

/// A "known" Rust type that can be directly marshalled across the FFI boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Primitive {
    String,
    Char,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
}

impl Primitive {
    pub fn from_type(ty: &Type) -> Option<Self> {
        let ident = match &*ty {
            Type::Path(path) => match path.path.get_ident() {
                Some(ident) => ident,
                None => return None,
            },

            _ => return None,
        };

        let prim = match &*ident.to_string() {
            "String" => Primitive::String,
            "char" => Primitive::Char,
            "i8" => Primitive::I8,
            "i16" => Primitive::I16,
            "i32" => Primitive::I32,
            "i64" => Primitive::I64,
            "u8" => Primitive::U8,
            "u16" => Primitive::U16,
            "u32" => Primitive::U32,
            "u64" => Primitive::U64,
            "f32" => Primitive::F32,
            "f64" => Primitive::F64,
            "bool" => Primitive::Bool,

            _ => return None,
        };

        Some(prim)
    }

    /// Generates the code for returning the final result of the function.
    pub fn generate_return_expr(
        &self,
        ret_val: &Ident,
        args: &mut Vec<TokenStream>,
    ) -> TokenStream {
        match self {
            Primitive::String => {
                // Generate the out param for the length of the string.
                let out_param = format_ident!("out_len");
                args.push(quote! {
                    #out_param: *mut i32
                });

                // Generate the code for
                quote! {
                    *#out_param = #ret_val
                        .len()
                        .try_into()
                        .expect("String length is too large for `i32`");

                    std::ffi::CString::new(#ret_val)
                        .expect("Generated string contained a null byte")
                        .into_raw()
                }
            }

            // Cast the bool to a `u8` in order to pass it to C# as a numeric value.
            Primitive::Bool => quote! {
                #ret_val as u8
            },

            // All other primitive types are ABI-compatible with a corresponding C# type, and
            // require no extra processing to be returned.
            _ => quote! { #ret_val },
        }
    }
}
