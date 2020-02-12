use crate::generate::func::*;
use cs_bindgen_shared::*;
use proc_macro2::TokenStream;
use quote::*;

pub fn quote_struct_binding(bindgen_struct: &BindgenStruct) -> TokenStream {
    let ident = bindgen_struct.ident();
    let drop_fn = bindgen_struct.drop_fn_ident();
    quote! {
        public unsafe partial class #ident : IDisposable
        {
            private void* _handle;

            public void Dispose()
            {
                if (_handle != null)
                {
                    __bindings.#drop_fn(_handle);
                    _handle = null;
                }
            }
        }
    }
}

pub fn quote_method_binding(item: &Method) -> TokenStream {
    let struct_ident = item.strct.ident();

    let wrapper_fn = if item.is_constructor() {
        quote_constructor(&item)
    } else {
        quote_wrapper_fn(&item.method, &item.binding_ident())
    };

    quote! {
        partial class #struct_ident
        {
            #wrapper_fn
        }
    }
}

fn quote_constructor(item: &Method) -> TokenStream {
    let args = quote_wrapper_args(&item.method);
    let struct_ident = item.strct.ident();
    let body = quote_wrapper_body(
        &item.method,
        &item.binding_ident(),
        &format_ident!("_handle"),
    );

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
