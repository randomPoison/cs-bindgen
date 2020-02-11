use crate::generate::func::*;
use cs_bindgen_shared::*;
use proc_macro2::TokenStream;
use quote::*;
use syn::Ident;

pub fn quote_struct_binding(bindgen_struct: &BindgenStruct) -> TokenStream {
    let ident = bindgen_struct.ident();
    quote! {
        public unsafe partial class #ident
        {
            private void* _handle;
        }
    }
}

pub fn quote_impl_binding(bindgen_impl: &BindgenImpl, dll_name: &str) -> TokenStream {
    let struct_ident = bindgen_impl.ty_ident();

    let methods = bindgen_impl.methods.iter().map(|method| {
        let raw_binding = quote_raw_binding(method, dll_name);
        let wrapper_fn = if method.is_constructor() {
            quote_constructor(&method, &struct_ident)
        } else {
            quote_wrapper_fn(&method)
        };

        quote! {
            #raw_binding
            #wrapper_fn
        }
    });

    quote! {
        partial class #struct_ident
        {
            #( #methods )*
        }
    }
}

fn quote_constructor(method: &BindgenFn, struct_ident: &Ident) -> TokenStream {
    let args = quote_wrapper_args(&method);
    let body = quote_wrapper_body(method, &format_ident!("_handle"));

    quote! {
        public #struct_ident(#args)
        {
            unsafe
            {
                #body
            }
        }
    }
}
