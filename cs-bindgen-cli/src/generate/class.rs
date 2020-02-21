use crate::generate::func::*;
use cs_bindgen_shared::{BindingStyle, Method, Schema, Struct};
use proc_macro2::TokenStream;
use quote::*;

pub fn quote_drop_fn(name: &str, dll_name: &str) -> TokenStream {
    let binding_ident = format_ident!("__cs_bindgen_drop__{}", name);
    let entry_point = binding_ident.to_string();
    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void #binding_ident(void* self);
    }
}

pub fn quote_struct(export: &Struct) -> TokenStream {
    match export.binding_style {
        BindingStyle::Handle => {
            let ident = format_ident!("{}", &*export.name);
            let drop_fn = format_ident!("__cs_bindgen_drop__{}", &*export.name);
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
    }
}

pub fn quote_method_binding(item: &Method) -> TokenStream {
    // Determine the name of the generated wrapper class based on the self type.
    let class_name = match &item.self_type {
        Schema::Struct(struct_) => &struct_.name,
        _ => todo!("Support methods for other named types"),
    };
    let class_ident = format_ident!("{}", &*class_name.name);

    // Use a heuristic to determine if the method should be treated as a constructor.
    //
    // TODO: Also support an explicit attribute to specify that a method should (or
    // should not) be treated as a constructor.
    let is_constructor = item.receiver.is_none() && item.output == item.self_type;

    let wrapper_fn = if is_constructor {
        let args = quote_args(item.inputs());
        let body = quote_wrapper_body(
            &*item.binding,
            item.inputs(),
            &item.output,
            &format_ident!("_handle"),
        );

        quote! {
            public #class_ident(#( #args, )*)
            {
                unsafe
                {
                    #body
                }
            }
        }
    } else {
        quote_wrapper_fn(&*item.name, &*item.binding, item.inputs(), &item.output)
    };

    quote! {
        partial class #class_ident
        {
            #wrapper_fn
        }
    }
}
