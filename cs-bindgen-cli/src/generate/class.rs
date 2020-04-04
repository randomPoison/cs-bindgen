use crate::generate::{binding, func::*, TypeMap};
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
        internal static extern void #binding_ident(void* self);
    }
}

/// Quotes the pointer type used for handles, i.e. `void*`.
pub fn quote_handle_ptr() -> TokenStream {
    quote! { void* }
}

pub fn quote_handle_type(export: &NamedType) -> TokenStream {
    let ident = format_ident!("{}", &*export.name);
    let drop_fn = format_ident!("__cs_bindgen_drop__{}", &*export.name);
    let raw_repr = binding::raw_ident(&export.name);

    let from_raw = binding::from_raw_fn_ident();
    let into_raw = binding::into_raw_fn_ident();

    let raw_conversions = binding::wrap_bindings(quote! {
        internal static #ident #from_raw(#raw_repr raw)
        {
            return new #ident(raw);
        }

        internal static #raw_repr #into_raw(#ident self)
        {
            return new #raw_repr(self);
        }
    });

    quote! {
        public unsafe partial class #ident : IDisposable
        {
            internal void* _handle;

            internal #ident(#raw_repr raw)
            {
                _handle = raw.Handle;
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

        [StructLayout(LayoutKind.Explicit)]
        internal unsafe struct #raw_repr
        {
            [FieldOffset(0)]
            public void* Handle;

            public #raw_repr(#ident orig)
            {
                this.Handle = orig._handle;
            }
        }

        #raw_conversions
    }
}

pub fn quote_method_binding(item: &Method, type_map: &TypeMap) -> TokenStream {
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
        let binding = format_ident!("{}", &*item.binding);
        let args = quote_args(item.inputs(), type_map);
        let invoke_args = quote_invoke_args(item.inputs());

        let invoke = fold_fixed_blocks(
            quote! { _handle = __bindings.#binding(#invoke_args).Handle; },
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
            item.output.as_ref(),
            type_map,
        )
    } else {
        quote_wrapper_fn(
            &*item.name,
            &*item.binding,
            None,
            item.inputs(),
            item.output.as_ref(),
            type_map,
        )
    };

    quote! {
        partial class #class_ident
        {
            #wrapper_fn
        }
    }
}
