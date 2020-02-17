use cs_bindgen_shared::schematic::TypeName;
use proc_macro2::TokenStream;
use quote::*;

pub fn quote_drop_fn(type_name: &TypeName, dll_name: &str) -> TokenStream {
    let binding_ident = format_ident!("__cs_bindgen_drop__{}", &*type_name.name);
    let entry_point = binding_ident.to_string();
    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void #binding_ident(void* self);
    }
}

pub fn quote_class(type_name: &TypeName) -> TokenStream {
    let ident = format_ident!("{}", &*type_name.name);
    let drop_fn = format_ident!("__cs_bindgen_drop__{}", &*type_name.name);
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

/*
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
*/
