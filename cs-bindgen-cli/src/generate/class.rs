use crate::generate::func::*;
use cs_bindgen_shared::{BindingStyle, Method, Schema, Struct};
use proc_macro2::TokenStream;
use quote::*;
use syn::Ident;

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
            quote_handle_type(&ident, &drop_fn)
        }

        BindingStyle::Value => unimplemented!("Pass struct by value"),
    }
}

fn quote_handle_type(name: &Ident, drop_fn: &Ident) -> TokenStream {
    quote! {
        public unsafe partial class #name : IDisposable
        {
            private void* _handle;

            internal #name(void* handle)
            {
                _handle = handle;
            }

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

    // Generate the right type of function for the exported method. There are three options:
    //
    // * A constructor.
    // * A non-static method.
    // * A static method.
    let wrapper_fn = if is_constructor {
        let binding = format_ident!("{}", &*item.binding);
        let args = quote_args(item.inputs());
        let invoke_args = quote_invoke_args(item.inputs());

        let invoke = fold_fixed_blocks(
            quote! { _handle = __bindings.#binding(#invoke_args); },
            item.inputs(),
        );

        quote! {
            public #class_ident(#( #args ),*)
            {
                unsafe
                {
                    #invoke
                }
            }
        }
    } else if let Some(_style) = &item.receiver {
        // TODO: Correctly handle `self` receivers. `&self` and `&mut self` are handled
        // correctly by passing the handle pointer directly, but in order to handle
        // `self` we'll need some concept of "consuming" the handle. Likely this will
        // meaning setting the handle to `null` after calling the function.
        quote_wrapper_fn(
            &*item.name,
            &*item.binding,
            Some(quote! { this._handle }),
            item.inputs(),
            &item.output,
        )
    } else {
        quote_wrapper_fn(
            &*item.name,
            &*item.binding,
            None,
            item.inputs(),
            &item.output,
        )
    };

    quote! {
        partial class #class_ident
        {
            #wrapper_fn
        }
    }
}
