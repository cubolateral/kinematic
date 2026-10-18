use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ImplItem, ItemImpl, Pat, ReturnType, Type, parse_macro_input};

pub fn expand(input: TokenStream) -> TokenStream {
    let mut block = parse_macro_input!(input as ItemImpl);
    let result = generate(&mut block);

    match result {
        Ok(bindings) => quote!(#block #bindings).into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn generate(block: &mut ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    if block.trait_.is_some() || !block.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &block.self_ty,
            "Lua methods require a concrete inherent impl.",
        ));
    }

    let handler = &block.self_ty;
    let mut entries = Vec::new();

    for item in &mut block.items {
        let ImplItem::Fn(method) = item else { continue };

        if !method.attrs.iter().any(|a| a.path().is_ident("lua")) {
            continue;
        }

        method.attrs.retain(|a| !a.path().is_ident("lua"));

        let signature = &method.sig;

        if signature.asyncness.is_some()
            || matches!(signature.safety, syn::Safety::Unsafe(_))
            || !signature.generics.params.is_empty()
        {
            return Err(syn::Error::new_spanned(
                signature,
                "Lua methods cannot be async, unsafe, or generic.",
            ));
        }

        let mut inputs = signature.inputs.iter();

        match inputs.next() {
            Some(FnArg::Receiver(receiver))
                if matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, None))
                    && receiver.mutability.is_none() =>
            {
                ()
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    signature,
                    "Lua methods require &self.",
                ));
            }
        }

        let mut names = Vec::new();
        let mut annotations = String::new();
        let mut conversions = Vec::new();

        for input in inputs {
            let FnArg::Typed(input) = input else {
                unreachable!()
            };

            let Pat::Ident(name) = &*input.pat else {
                return Err(syn::Error::new_spanned(
                    input,
                    "Lua arguments require simple names.",
                ));
            };

            let Type::Path(path) = &*input.ty else {
                return Err(syn::Error::new_spanned(
                    &input.ty,
                    "Unsupported Lua argument type.",
                ));
            };

            let ty = path.path.segments.last().unwrap().ident.to_string();
            let lua_type = match ty.as_str() {
                "String" => "string",
                "bool" => "boolean",
                "f32" | "f64" => "number",
                "i32" | "u32" | "i64" | "u64" => "integer",
                _ => {
                    return Err(syn::Error::new_spanned(
                        &input.ty,
                        "Supported Lua arguments are String, bool, f32, f64, i32, u32, i64, and u64.",
                    ));
                }
            };
            let ident = &name.ident;
            let rust_type = &input.ty;
            let compatible = match ty.as_str() {
                "String" => quote!(matches!(value, mlua::Value::String(_))),
                "bool" => quote!(matches!(value, mlua::Value::Boolean(_))),
                "f32" | "f64" => quote!(matches!(
                    value,
                    mlua::Value::Integer(_) | mlua::Value::Number(_)
                )),
                _ => quote!(
                    matches!(value, mlua::Value::Integer(_))
                        || matches!(value, mlua::Value::Number(v) if v.is_finite() && v.fract() == 0.0)
                ),
            };
            let validate = if ty == "f32" || ty == "f64" {
                quote!(if !#ident.is_finite() { return Err(mlua::Error::runtime("Native method arguments must be finite.")); })
            } else {
                quote!()
            };

            conversions.push(quote! {
                let value = values.next().unwrap();
                if !(#compatible) { return Err(mlua::Error::runtime("Incompatible native method argument type.")); }

                let #ident = <#rust_type as mlua::FromLua>::from_lua(value, lua)?;
                #validate
            });

            annotations.push_str(&format!("---@param {} {lua_type}\n", name.ident));
            names.push(&name.ident);
        }

        let ReturnType::Type(_, result) = &signature.output else {
            return Err(syn::Error::new_spanned(
                signature,
                "Lua animation methods must return Task.",
            ));
        };

        if !matches!(&**result, Type::Path(path) if path.path.segments.last().is_some_and(|s| s.ident == "Task"))
        {
            return Err(syn::Error::new_spanned(
                result,
                "Lua animation methods must return Task.",
            ));
        }

        let name = &signature.ident;
        let documentation = crate::documentation(&method.attrs);
        let key = name.to_string();
        let count = names.len();

        annotations.push_str(&format!(
            "---@return Kinematic.Animation\nfunction Native:{key}({}) end\n",
            names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));

        entries.push(quote! {
            crate::scripting::methods::LuaMethod {
                name: #key,
                annotation: #annotations,
                documentation: #documentation,
                invoke: |lua, object, args| {
                    use crate::core::objects::ObjectHandler;
                    use mlua::IntoLua;
                    if args.len() != #count { return Err(mlua::Error::runtime("Incorrect number of native method arguments.")); }

                    let mut values = args.into_iter();
                    #(#conversions)*
                    let runtime = object.runtime()?;
                    runtime.construction()?;
                    let handler = <<#handler as ObjectHandler>::Object as crate::core::objects::Object>::handler(
                        runtime.world.borrow().clone(), object.entity, runtime.animator.borrow().clone());
                    let task = crate::scripting::runtime::engine(|| handler.#name(#(#names),*))?;
                    crate::scripting::animation::AnimationDefinition::from_task(object.clone(), task)?.into_lua(lua)
                },
            }
        });
    }

    Ok(quote! {
        impl #handler {
            pub(crate) fn lua_methods() -> &'static [crate::scripting::methods::LuaMethod] {
                &[#(#entries),*]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_are_opt_in_and_reject_unsupported_signatures() {
        let mut block: ItemImpl = syn::parse_quote! {
            impl Handler {
                fn internal(&self) {}

                #[lua]
                fn increment(&self, amount: f32) -> Task { unimplemented!() }
            }
        };
        let bindings = generate(&mut block).unwrap().to_string();

        assert!(bindings.contains("increment") && bindings.contains("amount number"));
        assert!(!bindings.contains("internal"));

        for block in [
            quote!(impl Handler { #[lua] fn invalid(&mut self) -> Task {} }),
            quote!(impl Handler { #[lua] fn invalid(&self, value: Vec<f32>) -> Task {} }),
            quote!(impl Handler { #[lua] fn invalid<T>(&self, value: T) -> Task {} }),
            quote!(impl Handler { #[lua] fn invalid(&self) -> String {} }),
        ] {
            assert!(generate(&mut syn::parse2(block).unwrap()).is_err());
        }
    }
}
