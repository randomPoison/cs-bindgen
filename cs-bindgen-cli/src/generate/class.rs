//! Code generation for exported named types that are marshaled as handles.

use crate::generate::{binding, func, TypeMap};
use cs_bindgen_shared::{Method, NamedType, Schema};
use proc_macro2::TokenStream;
use quote::*;

pub fn quote_drop_fn(export: &NamedType, dll_name: &str) -> TokenStream {
    let binding_ident = format_ident!("__cs_bindgen_drop__{}", &*export.name);
    let entry_point = binding_ident.to_string();
    quote! {
        [DllImport(
            #dll_name,
            EntryPoint = #entry_point,
            CallingConvention = CallingConvention.Cdecl)]
        internal static extern void #binding_ident(IntPtr self);
    }
}

/// Quotes the pointer type used for handles, i.e. `IntPtr`.
pub fn quote_handle_ptr() -> TokenStream {
    quote! { IntPtr }
}

pub fn quote_handle_type(export: &NamedType) -> TokenStream {
    let ident = format_ident!("{}", &*export.name);
    let drop_fn = format_ident!("__cs_bindgen_drop__{}", &*export.name);
    let raw_repr = quote_handle_ptr();

    let from_raw = binding::from_raw_fn_ident();
    let into_raw = binding::into_raw_fn_ident();

    let raw_conversions = binding::wrap_bindings(quote! {
        internal static void #from_raw(#raw_repr raw, out #ident result)
        {
            result = new #ident(raw);
        }

        internal static void #into_raw(#ident value, out #raw_repr result)
        {
            result = value._handle;
        }
    });

    quote! {
        public unsafe partial class #ident : IDisposable
        {
            internal IntPtr _handle;

            internal #ident(#raw_repr raw)
            {
                _handle = raw;
            }

            public void Dispose()
            {
                if (_handle != IntPtr.Zero)
                {
                    __bindings.#drop_fn(_handle);
                    _handle = IntPtr.Zero;
                }
            }
        }

        #raw_conversions
    }
}

pub fn quote_method_binding(item: &Method, types: &TypeMap) -> TokenStream {
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
    let is_constructor = item.receiver.is_none() && item.output.as_ref() == Some(&item.self_type);

    // Generate the right type of function for the exported method. There are three options:
    //
    // * A constructor.
    // * A non-static method.
    // * A static method.
    let wrapper_fn = if is_constructor {
        let args = func::quote_args(item.inputs(), types);
        let body = func::quote_wrapper_body(
            &item.binding,
            None,
            &item.inputs().collect::<Vec<_>>(),
            Some(&quote! { this._handle }),
            types,
        );

        quote! {
            public #class_ident(#( #args ),*)
            {
                unsafe {
                    #body
                }
            }
        }
    } else if let Some(_style) = &item.receiver {
        // TODO: Correctly handle `self` receivers. `&self` and `&mut self` are handled
        // correctly by passing the handle pointer directly, but in order to handle
        // `self` we'll need some concept of "consuming" the handle. Likely this will
        // meaning setting the handle to `null` after calling the function.
        func::quote_wrapper_fn(
            &*item.name,
            &*item.binding,
            Some(quote! { this._handle }),
            item.inputs(),
            item.output.as_ref(),
            types,
        )
    } else {
        func::quote_wrapper_fn(
            &*item.name,
            &*item.binding,
            None,
            item.inputs(),
            item.output.as_ref(),
            types,
        )
    };

    quote! {
        partial class #class_ident
        {
            #wrapper_fn
        }
    }
}
