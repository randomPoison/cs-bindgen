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

pub fn quote_method_binding(item: &Method) -> TokenStream {
    let struct_ident = item.strct.ident();

    let wrapper_fn = if item.is_constructor() {
        quote_constructor(&item.method, &struct_ident)
    } else {
        quote_wrapper_fn(&item.method)
    };

    quote! {
        partial class #struct_ident
        {
            #wrapper_fn
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
