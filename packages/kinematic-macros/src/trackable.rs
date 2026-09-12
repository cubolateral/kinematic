use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Expr, Fields, Lit, UnOp, parse_macro_input, spanned::Spanned};

#[derive(Default)]
struct TrackArgs {
    min: Option<Expr>,
    max: Option<Expr>,
}

fn track_args(field: &syn::Field) -> syn::Result<Option<TrackArgs>> {
    let Some(attribute) = field
        .attrs
        .iter()
        .find(|attribute| attribute.path().is_ident("track"))
    else {
        return Ok(None);
    };
    let mut args = TrackArgs::default();
    if matches!(attribute.meta, syn::Meta::Path(_)) {
        return Ok(Some(args));
    }
    attribute.parse_nested_meta(|meta| {
        let target = if meta.path.is_ident("min") {
            &mut args.min
        } else if meta.path.is_ident("max") {
            &mut args.max
        } else {
            return Err(meta.error("Expected `min` or `max`."));
        };
        if target.is_some() {
            return Err(meta.error("Duplicate track limit."));
        }
        *target = Some(meta.value()?.parse()?);
        Ok(())
    })?;
    Ok(Some(args))
}

fn numeric_literal(expr: &Expr, integer: bool, unsigned: bool) -> syn::Result<f64> {
    let (negative, literal) = match expr {
        Expr::Lit(expr) => (false, &expr.lit),
        Expr::Unary(expr) if matches!(expr.op, UnOp::Neg(_)) => {
            let Expr::Lit(operand) = expr.expr.as_ref() else {
                return Err(syn::Error::new(
                    expr.span(),
                    "Track limits must be numeric literals.",
                ));
            };
            (true, &operand.lit)
        }
        _ => {
            return Err(syn::Error::new(
                expr.span(),
                "Track limits must be numeric literals.",
            ));
        }
    };

    if unsigned && negative {
        return Err(syn::Error::new(
            expr.span(),
            "A `u32` track limit cannot be negative.",
        ));
    }

    let value = match literal {
        Lit::Int(value) => value.base10_parse::<f64>(),
        Lit::Float(value) if !integer => value.base10_parse::<f64>(),
        _ => {
            return Err(syn::Error::new(
                expr.span(),
                "This track type requires integer limits.",
            ));
        }
    }
    .map_err(|error| syn::Error::new(expr.span(), error))?;

    Ok(if negative { -value } else { value })
}

fn limits_tokens(field: &syn::Field, args: &TrackArgs) -> syn::Result<proc_macro2::TokenStream> {
    let field_type = type_name(&field.ty);
    let Some(kind @ ("f32" | "i32" | "u32")) = field_type.as_deref() else {
        if args.min.is_some() || args.max.is_some() {
            return Err(syn::Error::new(
                field.ty.span(),
                "Track limits are only supported for `f32`, `i32`, and `u32`.",
            ));
        }
        return Ok(quote!(kinematic::core::TrackLimits::None));
    };

    if args.min.is_none() && args.max.is_none() {
        return Ok(quote!(kinematic::core::TrackLimits::None));
    }

    let integer = kind != "f32";
    let unsigned = kind == "u32";
    let min_value = args
        .min
        .as_ref()
        .map(|value| numeric_literal(value, integer, unsigned))
        .transpose()?;
    let max_value = args
        .max
        .as_ref()
        .map(|value| numeric_literal(value, integer, unsigned))
        .transpose()?;

    if let (Some(min), Some(max)) = (min_value, max_value)
        && min > max
    {
        return Err(syn::Error::new(
            field.span(),
            "Track `min` cannot be greater than `max`.",
        ));
    }
    if integer {
        let max = if unsigned {
            u32::MAX as f64
        } else {
            i32::MAX as f64
        };
        let min = if unsigned { 0.0 } else { i32::MIN as f64 };
        for (expr, value) in [
            args.min.as_ref().zip(min_value),
            args.max.as_ref().zip(max_value),
        ]
        .into_iter()
        .flatten()
        {
            if value < min || value > max {
                return Err(syn::Error::new(
                    expr.span(),
                    "Track limit is outside the field type's range.",
                ));
            }
        }
    }

    let min = args
        .min
        .as_ref()
        .map_or_else(|| quote!(None), |value| quote!(Some(#value)));
    let max = args
        .max
        .as_ref()
        .map_or_else(|| quote!(None), |value| quote!(Some(#value)));
    let variant = format_ident!("{}", kind.to_uppercase());
    Ok(quote! {
        kinematic::core::TrackLimits::#variant {
            min: #min,
            max: #max,
        }
    })
}

fn type_fragment(identifier: &syn::Ident) -> String {
    identifier
        .to_string()
        .trim_start_matches("r#")
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();

            match characters.next() {
                Some(first) => first.to_uppercase().chain(characters).collect(),
                None => String::new(),
            }
        })
        .collect()
}

fn type_name(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = ty else {
        return None;
    };

    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

/// Generates object-handler fields and track metadata for a `Trackable` component.
pub fn derive_trackable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let handler_fields_name = format_ident!("__{}HandlerFields", struct_name);
    let tween_fields_trait = format_ident!("__Kinematic{}TweenFields", struct_name);
    let builder_component_trait = quote!(kinematic::core::objects::ObjectBuilderComponent);
    let handler_context_trait = quote!(kinematic::core::objects::HandlerContext);
    let object_trackable_trait = quote!(kinematic::core::objects::ObjectTrackable);
    let scene_world_type = quote!(kinematic::core::SceneWorld);
    let animator_handle_type = quote!(kinematic::core::AnimatorHandle);
    let track_handle_type = quote!(kinematic::core::TrackHandle);
    let track_id_type = quote!(kinematic::core::TrackId);
    let track_info_type = quote!(kinematic::core::TrackInfo);
    let trackable_info_type = quote!(kinematic::core::TrackableInfo);
    let trackable_trait = quote!(kinematic::core::Trackable);
    let track_value_type_trait = quote!(kinematic::core::TrackValueType);
    let tween_type = quote!(kinematic::core::Tween);
    let track_property_type = quote!(kinematic::core::TrackProperty);
    let vector3_type = quote!(kinematic::core::types::Vector3);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("Trackable only supports structs with named fields."),
        },
        _ => panic!("Trackable can only be derived for structs."),
    };

    let tracked_fields: Vec<_> = match fields
        .iter()
        .map(|field| {
            track_args(field).and_then(|args| {
                args.map(|args| limits_tokens(field, &args).map(|limits| (field, limits)))
                    .transpose()
            })
        })
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(fields) => fields.into_iter().flatten().collect(),
        Err(error) => return error.into_compile_error().into(),
    };
    let count = tracked_fields.len();
    let mut type_assertions = Vec::with_capacity(count);
    let mut track_entries = Vec::with_capacity(count);
    let mut handle_fields = Vec::with_capacity(count);
    let mut handle_initializers = Vec::with_capacity(count);
    let mut tween_fns = Vec::with_capacity(count);
    let mut direct_fns = Vec::with_capacity(count * 2);
    let mut tween_trait_fns = Vec::with_capacity(count);
    let mut tween_impl_fns = Vec::with_capacity(count);
    let mut property_constants = Vec::with_capacity(count);
    let mut builder_setters = Vec::with_capacity(fields.len());

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let field_visibility = &field.vis;
        let setter_trait = format_ident!(
            "__Kinematic{}{}BuilderSetter",
            struct_name,
            type_fragment(field_ident)
        );

        let (setter_generic, setter_value_type, setter_value) =
            if matches!(type_name(field_ty).as_deref(), Some("Quad" | "String")) {
                (
                    quote!(<Value: Into<#field_ty>>),
                    quote!(Value),
                    quote!(value.into()),
                )
            } else {
                (quote!(), quote!(#field_ty), quote!(value))
            };
        let assignment = tracked_fields
            .iter()
            .position(|(tracked, _)| std::ptr::eq(*tracked, field))
            .map_or_else(
                || quote! {
                    <T as #builder_component_trait<#struct_name>>::component_mut(
                        &mut self,
                    ).#field_ident = #setter_value;
                },
                |id| {
                    let id = id as u32;
                    quote! {
                        let value = #setter_value;
                        let value = <#struct_name as #trackable_trait>::track(#id).clamp(
                            <#field_ty as #track_value_type_trait>::into_track_value(value),
                        );
                        <T as #builder_component_trait<#struct_name>>::component_mut(
                            &mut self,
                        ).#field_ident = <#field_ty as #track_value_type_trait>::from_track_value(value)
                            .expect("Track limits must preserve the field value type.");
                    }
                },
            );
        builder_setters.push(quote! {
            #[doc(hidden)]
            #field_visibility trait #setter_trait: Sized {
                fn #field_ident #setter_generic (self, value: #setter_value_type) -> Self;
            }

            #[doc(hidden)]
            impl<T> #setter_trait for T
            where
                T: #builder_component_trait<#struct_name>,
            {
                fn #field_ident #setter_generic (mut self, value: #setter_value_type) -> Self {
                    #assignment
                    self
                }
            }
        });

        let component_fields: &[&str] = match type_name(field_ty).as_deref() {
            Some("Vector2") => &["x", "y"],
            Some("Vector3") => &["x", "y", "z"],
            Some("Quad") => &["a", "b", "c", "d"],
            Some("Color") => &["r", "g", "b", "a"],
            _ => &[],
        };

        for component_field in component_fields {
            let method_name = format_ident!("{}_{}", field_ident, component_field);
            let component_field = format_ident!("{}", component_field);
            let setter_trait = format_ident!(
                "__Kinematic{}{}BuilderSetter",
                struct_name,
                type_fragment(&method_name)
            );

            builder_setters.push(quote! {
                #[doc(hidden)]
                #field_visibility trait #setter_trait: Sized {
                    fn #method_name(self, value: f32) -> Self;
                }

                #[doc(hidden)]
                impl<T> #setter_trait for T
                where
                    T: #builder_component_trait<#struct_name>,
                {
                    fn #method_name(mut self, value: f32) -> Self {
                        <T as #builder_component_trait<#struct_name>>::component_mut(
                            &mut self,
                        ).#field_ident.#component_field = value;
                        self
                    }
                }
            });
        }
    }

    for (id, (field, limits)) in tracked_fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let id = id as u32;
        let field_name = field_ident.to_string();
        let property_name = format_ident!("{}_property", field_ident);
        let from_method_name = format_ident!("{}_from", field_ident);
        let get_method_name = format_ident!("get_{}", field_ident);
        let set_method_name = format_ident!("set_{}", field_ident);

        type_assertions.push(quote! {
            const _: fn() = {
                fn assert_track_value_type<T: #track_value_type_trait>() {}
                assert_track_value_type::<#field_ty>
            };
        });

        track_entries.push(quote! {
            #track_info_type {
                id: #id,
                name: #field_name,
                limits: #limits,
                get: |world, entity| {
                    <#struct_name as #trackable_trait>::track(#id).clamp(
                        <#field_ty as #track_value_type_trait>::into_track_value(
                        world.get::<&#struct_name>(entity).unwrap().#field_ident.clone()
                        )
                    )
                },
                set: |world, entity, value| {
                    let value = <#struct_name as #trackable_trait>::track(#id).clamp(value);
                    if let Some(value) = <#field_ty as #track_value_type_trait>::from_track_value(value) {
                        world.get::<&mut #struct_name>(entity).unwrap().#field_ident = value;
                    }
                },
            }
        });

        handle_fields.push(quote! {
            #field_ident: #track_handle_type<#field_ty>,
        });

        handle_initializers.push(quote! {
            #field_ident: #track_handle_type::new(
                std::rc::Rc::clone(&world),
                entity,
                std::any::TypeId::of::<#struct_name>(),
                <#struct_name as #trackable_trait>::track(#id),
                |world, entity| {
                    world
                        .get::<&#struct_name>(entity)
                        .unwrap()
                        .#field_ident
                        .clone()
                },
                |world, entity, value| {
                    let value = <#struct_name as #trackable_trait>::track(#id).clamp(
                        <#field_ty as #track_value_type_trait>::into_track_value(value),
                    );
                    let value = <#field_ty as #track_value_type_trait>::from_track_value(value)
                        .expect("Track limits must preserve the field value type.");
                    let mut component = world
                        .get::<&mut #struct_name>(entity)
                        .unwrap();
                    let old_value = component.#field_ident.clone();
                    component.#field_ident = value;
                    old_value
                },
                animator.clone(),
            ),
        });

        direct_fns.push(quote! {
            pub fn #get_method_name(&self) -> #field_ty {
                self.#field_ident.get()
            }

            pub fn #set_method_name<Value: Into<#field_ty>>(&self, value: Value) {
                self.#field_ident.set_direct(value.into());
            }
        });

        match type_name(field_ty).as_deref() {
            Some(name @ ("Vector2" | "Vector3" | "Quad" | "Color")) => {
                let components: &[&str] = match name {
                    "Vector2" | "Vector3" => &["x", "y", "z"],
                    "Quad" => &["a", "b", "c", "d"],
                    _ => &["r", "g", "b", "a"],
                };

                for component in components
                    .iter()
                    .copied()
                    .filter(|component| !matches!((name, *component), ("Vector2", "z")))
                {
                    let component_method_name = format_ident!("{}_{}", get_method_name, component);
                    let component_set_method_name =
                        format_ident!("{}_{}", set_method_name, component);
                    let component_field = format_ident!("{}", component);

                    direct_fns.push(quote! {
                        pub fn #component_method_name(&self) -> f32 {
                            self.#field_ident.get().#component_field
                        }

                        pub fn #component_set_method_name(&self, value: f32) {
                            let mut component = self.#field_ident.get();
                            component.#component_field = value;
                            self.#field_ident.set_direct(component);
                        }
                    });
                }
            }
            _ => {}
        }

        let (value_generic, value_type, from_generic, from_type, to_type) =
            if matches!(type_name(field_ty).as_deref(), Some("Quad" | "String")) {
                (
                    quote!(<Value: Into<#field_ty>>),
                    quote!(Value),
                    quote!(<FromValue: Into<#field_ty>, ToValue: Into<#field_ty>>),
                    quote!(FromValue),
                    quote!(ToValue),
                )
            } else {
                let input = quote!(<#field_ty as #track_value_type_trait>::Input);
                (quote!(), input.clone(), quote!(), input.clone(), input)
            };
        tween_fns.push(quote! {
            pub fn #field_ident #value_generic (
                &self,
                value: #value_type,
            ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                self.#field_ident.animate::< <Next as #handler_context_trait>::Object >(value.into())
            }

            pub fn #from_method_name #from_generic (
                &self,
                from: #from_type,
                to: #to_type,
            ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                self.#field_ident.animate_from::< <Next as #handler_context_trait>::Object >(from.into(), to.into())
            }
        });
        tween_trait_fns.push(quote! {
            fn #field_ident #value_generic (
                self,
                value: #value_type,
            ) -> Self;

            fn #from_method_name #from_generic (
                self,
                from: #from_type,
                to: #to_type,
            ) -> Self;
        });
        tween_impl_fns.push(quote! {
            fn #field_ident #value_generic (
                self,
                value: #value_type,
            ) -> Self {
                self.set_track::<#field_ty>(
                    std::any::TypeId::of::<#struct_name>(),
                    <#struct_name as #trackable_trait>::track(#id),
                    value.into(),
                )
            }

            fn #from_method_name #from_generic (
                self,
                from: #from_type,
                to: #to_type,
            ) -> Self {
                self.animate_from(
                    #struct_name::#property_name(),
                    from.into(),
                    to.into(),
                )
            }
        });

        property_constants.push(quote! {
            pub fn #property_name() -> #track_property_type<#field_ty> {
                #track_property_type::new(
                    std::any::TypeId::of::<#struct_name>(),
                    <#struct_name as #trackable_trait>::track(#id),
                    |world, entity| {
                        world.get::<&#struct_name>(entity).unwrap().#field_ident.clone()
                    },
                    |world, entity, value| {
                        let value = <#struct_name as #trackable_trait>::track(#id).clamp(
                            <#field_ty as #track_value_type_trait>::into_track_value(value),
                        );
                        let value = <#field_ty as #track_value_type_trait>::from_track_value(value)
                            .expect("Track limits must preserve the field value type.");
                        let mut component = world.get::<&mut #struct_name>(entity).unwrap();
                        let old_value = component.#field_ident.clone();
                        component.#field_ident = value;
                        old_value
                    },
                )
            }
        });

        if matches!(
            type_name(field_ty).as_deref(),
            Some("f32" | "Vector2" | "Vector3")
        ) {
            let by_method_name = format_ident!("{}_by", field_ident);

            tween_fns.push(quote! {
                pub fn #by_method_name(
                    &self,
                    delta: #field_ty,
                ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                    self.#field_ident.update_for::< <Next as #handler_context_trait>::Object >(
                        |value| value + delta,
                    )
                }
            });
            tween_trait_fns.push(quote! {
                fn #by_method_name(self, delta: #field_ty) -> Self;
            });
            tween_impl_fns.push(quote! {
                fn #by_method_name(self, delta: #field_ty) -> Self {
                    self.update_track(
                        std::any::TypeId::of::<#struct_name>(),
                        <#struct_name as #trackable_trait>::track(#id),
                        |value: #field_ty| value + delta,
                    )
                }
            });
        }

        match type_name(field_ty).as_deref() {
            Some(name @ ("Vector2" | "Vector3" | "Quad")) => {
                let axes: &[&str] = match name {
                    "Vector3" => &["x", "y", "z"],
                    "Quad" => &["a", "b", "c", "d"],
                    _ => &["x", "y"],
                };
                for suffix in axes {
                    let method_name = format_ident!("{}_{}", field_ident, suffix);
                    let from_method_name = format_ident!("{}_from", method_name);
                    let component_field = format_ident!("{}", suffix);

                    tween_fns.push(quote! {
                        pub fn #method_name(
                            &self,
                            value: f32,
                        ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                            let mut component = self.#field_ident.get();
                            component.#component_field = value;
                            self.#field_ident.animate::< <Next as #handler_context_trait>::Object >(component)
                        }

                        pub fn #from_method_name(
                            &self,
                            from: f32,
                            to: f32,
                        ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                            let mut from_component = self.#field_ident.get();
                            from_component.#component_field = from;
                            let mut to_component = self.#field_ident.get();
                            to_component.#component_field = to;

                            self.#field_ident.animate_from::< <Next as #handler_context_trait>::Object >(
                                from_component,
                                to_component,
                            )
                        }
                    });
                    tween_trait_fns.push(quote! {
                        fn #method_name(self, value: f32) -> Self;
                    });
                    tween_impl_fns.push(quote! {
                        fn #method_name(self, value: f32) -> Self {
                            self.update_track(
                                std::any::TypeId::of::<#struct_name>(),
                                <#struct_name as #trackable_trait>::track(#id),
                                |mut component: #field_ty| {
                                component.#component_field = value;
                                component
                                },
                            )
                        }
                    });

                    let by_method_name = format_ident!("{}_by", method_name);
                    tween_fns.push(quote! {
                        pub fn #by_method_name(
                            &self,
                            delta: f32,
                        ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                            self.#field_ident.update_for::< <Next as #handler_context_trait>::Object >(
                                |mut component| {
                                    component.#component_field += delta;
                                    component
                                },
                            )
                        }
                    });
                    tween_trait_fns.push(quote! {
                        fn #by_method_name(self, delta: f32) -> Self;
                    });
                    tween_impl_fns.push(quote! {
                        fn #by_method_name(self, delta: f32) -> Self {
                            self.update_track(
                                std::any::TypeId::of::<#struct_name>(),
                                <#struct_name as #trackable_trait>::track(#id),
                                |mut component: #field_ty| {
                                    component.#component_field += delta;
                                    component
                                },
                            )
                        }
                    });
                }
            }
            Some("Color") => {
                for suffix in ["r", "g", "b", "a"] {
                    let method_name = format_ident!("{}_{}", field_ident, suffix);
                    let from_method_name = format_ident!("{}_from", method_name);
                    let component_field = format_ident!("{}", suffix);

                    tween_fns.push(quote! {
                        pub fn #method_name(
                            &self,
                            value: f32,
                        ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                            let mut component = self.#field_ident.get();
                            component.#component_field = value;
                            self.#field_ident.animate::< <Next as #handler_context_trait>::Object >(component)
                        }

                        pub fn #from_method_name(
                            &self,
                            from: f32,
                            to: f32,
                        ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                            let mut from_component = self.#field_ident.get();
                            from_component.#component_field = from;
                            let mut to_component = self.#field_ident.get();
                            to_component.#component_field = to;

                            self.#field_ident.animate_from::< <Next as #handler_context_trait>::Object >(
                                from_component,
                                to_component,
                            )
                        }
                    });
                    tween_trait_fns.push(quote! {
                        fn #method_name(self, value: f32) -> Self;
                    });
                    tween_impl_fns.push(quote! {
                        fn #method_name(self, value: f32) -> Self {
                            self.update_track(
                                std::any::TypeId::of::<#struct_name>(),
                                <#struct_name as #trackable_trait>::track(#id),
                                |mut component: #field_ty| {
                                    component.#component_field = value;
                                    component
                                },
                            )
                        }
                    });
                }
            }
            Some("Quaternion") if field_ident == "rotation" => {
                for (method, axis) in [
                    ("rotate_x", quote!(#vector3_type::X)),
                    ("rotate_y", quote!(#vector3_type::Y)),
                    ("rotate_z", quote!(#vector3_type::Z)),
                ] {
                    let method_name = format_ident!("{}", method);

                    tween_fns.push(quote! {
                        pub fn #method_name(
                            &self,
                            angle: f32,
                        ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                            self.#field_ident.rotate_for::< <Next as #handler_context_trait>::Object >(
                                #axis,
                                angle,
                            )
                        }
                    });
                    tween_trait_fns.push(quote! {
                        fn #method_name(self, angle: f32) -> Self;
                    });
                    tween_impl_fns.push(quote! {
                        fn #method_name(self, angle: f32) -> Self {
                            self.rotate_track(#struct_name::#property_name(), #axis, angle)
                        }
                    });
                }

                tween_fns.push(quote! {
                    pub fn rotate_axis(
                        &self,
                        axis: #vector3_type,
                        angle: f32,
                    ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                        self.#field_ident.rotate_for::< <Next as #handler_context_trait>::Object >(
                            axis,
                            angle,
                        )
                    }
                });
                tween_trait_fns.push(quote! {
                    fn rotate_axis(self, axis: #vector3_type, angle: f32) -> Self;
                });
                tween_impl_fns.push(quote! {
                    fn rotate_axis(self, axis: #vector3_type, angle: f32) -> Self {
                        self.rotate_track(#struct_name::#property_name(), axis, angle)
                    }
                });
            }
            _ => {}
        }
    }

    let tracks_ident = format_ident!("__{}_TRACKS", struct_name.to_string().to_uppercase());
    let expanded = quote! {
        #(#type_assertions)*
        #(#builder_setters)*

        /// Internal tracked-field layer used by generated object handlers.
        #[doc(hidden)]
        pub struct #handler_fields_name<Next> {
            #(#handle_fields)*
            next: Next,
        }

        impl<Next> std::ops::Deref for #handler_fields_name<Next> {
            type Target = Next;

            fn deref(&self) -> &Self::Target {
                &self.next
            }
        }

        impl<Next: #handler_context_trait> #handler_fields_name<Next> {
            #(#direct_fns)*
            #(#tween_fns)*
        }

        impl<Next: #handler_context_trait> #handler_context_trait for #handler_fields_name<Next> {
            type Object = <Next as #handler_context_trait>::Object;
        }

        /// Chained tracked fields available to tweens for this component.
        #[doc(hidden)]
        pub trait #tween_fields_trait: Sized {
            #(#tween_trait_fns)*
        }

        #[doc(hidden)]
        impl<ObjectType> #tween_fields_trait for #tween_type<ObjectType>
        where
            ObjectType: #object_trackable_trait<#struct_name>,
        {
            #(#tween_impl_fns)*
        }

        #[allow(non_upper_case_globals)]
        const #tracks_ident: [#track_info_type; #count] = [
            #(#track_entries),*
        ];

        impl #trackable_trait for #struct_name {
            type HandlerFields<Next: #handler_context_trait> = #handler_fields_name<Next>;

            fn handler_fields<Next: #handler_context_trait>(
                world: #scene_world_type,
                entity: hecs::Entity,
                animator: #animator_handle_type,
                next: Next,
            ) -> Self::HandlerFields<Next> {
                #handler_fields_name {
                    #(#handle_initializers)*
                    next,
                }
            }

            fn track(id: #track_id_type) -> &'static #track_info_type {
                &#tracks_ident[id as usize]
            }

            fn info() -> &'static #trackable_info_type {
                &Self::INFO
            }
        }

        impl #struct_name {
            #(#property_constants)*

            pub const INFO: #trackable_info_type = #trackable_info_type {
                name: stringify!(#struct_name),
                type_id: || std::any::TypeId::of::<#struct_name>(),
                get: || &#tracks_ident,
            };
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn parses_optional_track_limits() {
        let unbounded: syn::Field = parse_quote!(#[track] value: f32);
        let minimum: syn::Field = parse_quote!(#[track(min = 0.0)] value: f32);
        let maximum: syn::Field = parse_quote!(#[track(max = 100)] value: u32);

        assert!(limits_tokens(&unbounded, &track_args(&unbounded).unwrap().unwrap()).is_ok());
        assert!(limits_tokens(&minimum, &track_args(&minimum).unwrap().unwrap()).is_ok());
        assert!(limits_tokens(&maximum, &track_args(&maximum).unwrap().unwrap()).is_ok());
    }

    #[test]
    fn rejects_limits_for_other_track_types() {
        let field: syn::Field = parse_quote!(#[track(min = 0)] value: bool);
        let args = track_args(&field).unwrap().unwrap();

        assert!(limits_tokens(&field, &args).is_err());
    }

    #[test]
    fn rejects_reversed_limits() {
        let field: syn::Field = parse_quote!(#[track(min = 10, max = -10)] value: i32);
        let args = track_args(&field).unwrap().unwrap();

        assert!(limits_tokens(&field, &args).is_err());
    }
}
