use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

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
    let builder_component_trait = format_ident!("__Kinematic{}BuilderComponent", struct_name);
    let handler_context_trait = format_ident!("__Kinematic{}HandlerContext", struct_name);
    let object_trackable_trait = format_ident!("__Kinematic{}ObjectTrackable", struct_name);
    let scene_world_type = format_ident!("__Kinematic{}SceneWorld", struct_name);
    let animator_handle_type = format_ident!("__Kinematic{}AnimatorHandle", struct_name);
    let track_handle_type = format_ident!("__Kinematic{}TrackHandle", struct_name);
    let track_id_type = format_ident!("__Kinematic{}TrackId", struct_name);
    let track_info_type = format_ident!("__Kinematic{}TrackInfo", struct_name);
    let trackable_info_type = format_ident!("__Kinematic{}TrackableInfo", struct_name);
    let trackable_trait = format_ident!("__Kinematic{}Trackable", struct_name);
    let track_value_type_trait = format_ident!("__Kinematic{}TrackValueType", struct_name);
    let tween_type = format_ident!("__Kinematic{}Tween", struct_name);
    let track_property_type = format_ident!("__Kinematic{}TrackProperty", struct_name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("Trackable only supports structs with named fields."),
        },
        _ => panic!("Trackable can only be derived for structs."),
    };

    let tracked_fields: Vec<_> = fields
        .iter()
        .filter(|field| {
            field
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("track"))
        })
        .collect();
    let count = tracked_fields.len();
    let mut type_assertions = Vec::with_capacity(count);
    let mut track_entries = Vec::with_capacity(count);
    let mut handle_fields = Vec::with_capacity(count);
    let mut handle_initializers = Vec::with_capacity(count);
    let mut tween_fns = Vec::with_capacity(count);
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

        builder_setters.push(quote! {
            #[doc(hidden)]
            #field_visibility trait #setter_trait: Sized {
                fn #field_ident(self, value: #field_ty) -> Self;
            }

            impl<T> #setter_trait for T
            where
                T: #builder_component_trait<#struct_name>,
            {
                fn #field_ident(mut self, value: #field_ty) -> Self {
                    <T as #builder_component_trait<#struct_name>>::component_mut(
                        &mut self,
                    ).#field_ident = value;
                    self
                }
            }
        });
    }

    for (id, field) in tracked_fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let id = id as u32;
        let field_name = field_ident.to_string();
        let property_name = format_ident!("{}_property", field_ident);
        let from_method_name = format_ident!("{}_from", field_ident);

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
                get: |world, entity| {
                    <#field_ty as #track_value_type_trait>::into_track_value(
                        world.get::<&#struct_name>(entity).unwrap().#field_ident.clone()
                    )
                },
                set: |world, entity, value| {
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

        tween_fns.push(quote! {
            pub fn #field_ident(
                &self,
                value: <#field_ty as #track_value_type_trait>::Input,
            ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                self.#field_ident.animate::< <Next as #handler_context_trait>::Object >(value.into())
            }

            pub fn #from_method_name(
                &self,
                from: <#field_ty as #track_value_type_trait>::Input,
                to: <#field_ty as #track_value_type_trait>::Input,
            ) -> #tween_type<<Next as #handler_context_trait>::Object> {
                self.#field_ident.animate_from::< <Next as #handler_context_trait>::Object >(from.into(), to.into())
            }
        });
        tween_trait_fns.push(quote! {
            fn #field_ident(
                self,
                value: <#field_ty as #track_value_type_trait>::Input,
            ) -> Self;

            fn #from_method_name(
                self,
                from: <#field_ty as #track_value_type_trait>::Input,
                to: <#field_ty as #track_value_type_trait>::Input,
            ) -> Self;
        });
        tween_impl_fns.push(quote! {
            fn #field_ident(
                self,
                value: <#field_ty as #track_value_type_trait>::Input,
            ) -> Self {
                self.set_track::<#field_ty>(
                    std::any::TypeId::of::<#struct_name>(),
                    <#struct_name as #trackable_trait>::track(#id),
                    value.into(),
                )
            }

            fn #from_method_name(
                self,
                from: <#field_ty as #track_value_type_trait>::Input,
                to: <#field_ty as #track_value_type_trait>::Input,
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
                        let mut component = world.get::<&mut #struct_name>(entity).unwrap();
                        let old_value = component.#field_ident.clone();
                        component.#field_ident = value;
                        old_value
                    },
                )
            }
        });

        match type_name(field_ty).as_deref() {
            Some("Vector2") => {
                for suffix in ["x", "y"] {
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
            _ => {}
        }
    }

    let tracks_ident = format_ident!("__{}_TRACKS", struct_name.to_string().to_uppercase());
    let expanded = quote! {
        use crate::core::{
            SceneWorld as #scene_world_type,
            AnimatorHandle as #animator_handle_type,
            TrackHandle as #track_handle_type,
            TrackProperty as #track_property_type,
            TrackId as #track_id_type,
            TrackInfo as #track_info_type,
            Trackable as #trackable_trait,
            TrackableInfo as #trackable_info_type,
            TrackValueType as #track_value_type_trait,
            Tween as #tween_type,
            objects::{
                HandlerContext as #handler_context_trait,
                ObjectBuilderComponent as #builder_component_trait,
                ObjectTrackable as #object_trackable_trait,
            },
        };

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
