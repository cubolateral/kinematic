use quote::{format_ident, quote};
use syn::{Error, ItemFn, ReturnType, Safety, parse_macro_input};

/// Converts a scene-building function into a zero-argument scene factory.
pub fn scene(
    attribute: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if !attribute.is_empty() {
        return Error::new_spanned(
            proc_macro2::TokenStream::from(attribute),
            "The `scene` attribute does not accept arguments.",
        )
        .into_compile_error()
        .into();
    }

    let function = parse_macro_input!(input as ItemFn);

    if function.sig.constness.is_some()
        || function.sig.asyncness.is_some()
        || !matches!(function.sig.safety, Safety::Default)
        || function.sig.abi.is_some()
    {
        return Error::new_spanned(
            &function.sig,
            "A `scene` function must be a safe, synchronous Rust function.",
        )
        .into_compile_error()
        .into();
    }

    if !function.sig.generics.params.is_empty() || function.sig.generics.where_clause.is_some() {
        return Error::new_spanned(
            &function.sig.generics,
            "A `scene` function cannot have generic parameters or a where clause.",
        )
        .into_compile_error()
        .into();
    }

    if function.sig.inputs.len() != 1 {
        return Error::new_spanned(
            &function.sig.inputs,
            "A `scene` function must take a `&mut Scene` parameter.",
        )
        .into_compile_error()
        .into();
    }

    if function
        .sig
        .inputs
        .iter()
        .any(|argument| matches!(argument, syn::FnArg::Receiver(_)))
    {
        return Error::new_spanned(
            &function.sig.inputs,
            "A `scene` function cannot take a receiver.",
        )
        .into_compile_error()
        .into();
    }

    if !matches!(function.sig.output, ReturnType::Default) {
        return Error::new_spanned(
            &function.sig.output,
            "A `scene` function cannot return a value.",
        )
        .into_compile_error()
        .into();
    }

    let attributes = function.attrs;
    let visibility = function.vis;
    let name = function.sig.ident;
    let inputs = function.sig.inputs;
    let body = function.block;
    let builder_name = format_ident!("__KinematicSceneBuilder");

    quote! {
        #(#attributes)*
        #visibility fn #name() -> std::boxed::Box<dyn kinematic::core::SceneBuilder> {
            struct #builder_name;

            impl kinematic::core::SceneBuilder for #builder_name {
                fn build(&mut self, #inputs) #body
            }

            std::boxed::Box::new(#builder_name)
        }
    }
    .into()
}
