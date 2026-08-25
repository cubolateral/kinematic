use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn derive_object(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let object_name = &input.ident;
    let visibility = &input.vis;
    let builder_name = format_ident!("{}Builder", object_name);
    let handler_name = format_ident!("{}Handler", object_name);
    let builder_component_trait = format_ident!("__Kinematic{}BuilderComponent", object_name);
    let inspection_type = format_ident!("__Kinematic{}Inspection", object_name);
    let name_type = format_ident!("__Kinematic{}Name", object_name);
    let object_trait = format_ident!("__Kinematic{}Object", object_name);
    let object_handler_trait = format_ident!("__Kinematic{}ObjectHandler", object_name);
    let handler_root_type = format_ident!("__Kinematic{}HandlerRoot", object_name);
    let object_trackable_trait = format_ident!("__Kinematic{}ObjectTrackable", object_name);
    let scene_world_type = format_ident!("__Kinematic{}SceneWorld", object_name);
    let animator_handle_type = format_ident!("__Kinematic{}AnimatorHandle", object_name);
    let trackable_info_type = format_ident!("__Kinematic{}TrackableInfo", object_name);
    let trackable_trait = format_ident!("__Kinematic{}Trackable", object_name);
    let track_property_type = format_ident!("__Kinematic{}TrackProperty", object_name);
    let track_value_type_trait = format_ident!("__Kinematic{}TrackValueType", object_name);
    let tween_type = format_ident!("__Kinematic{}Tween", object_name);
    let draw_type = format_ident!("__Kinematic{}Draw", object_name);
    let vector_type = format_ident!("__Kinematic{}Vector2", object_name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => panic!("Object only supports structs with named fields."),
        },
        _ => panic!("Object can only be derived for structs."),
    };

    let mut component_accessors = Vec::with_capacity(fields.len());
    let mut trackable_infos = Vec::with_capacity(fields.len());
    let mut trackable_types = Vec::with_capacity(fields.len());
    let field_idents: Vec<_> = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect();
    let field_types: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let is_trackable = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("trackable"));

        if !is_trackable {
            continue;
        }

        trackable_infos.push(quote! {
            <#field_ty>::INFO
        });
        trackable_types.push(field_ty);

        component_accessors.push(quote! {
            impl #builder_component_trait<#field_ty> for #builder_name {
                fn component_mut(&mut self) -> &mut #field_ty {
                    &mut self.object.#field_ident
                }
            }
        });
    }

    let count = trackable_infos.len();
    let infos_ident = format_ident!("__{}_TRACKABLES", object_name.to_string().to_uppercase());
    let infos_fn_ident = format_ident!("__{}_trackables", object_name.to_string().to_lowercase());
    let mut handler_fields_type = quote!(#handler_root_type<#object_name>);

    for field_ty in trackable_types.iter().rev() {
        handler_fields_type = quote! {
            <#field_ty as #trackable_trait>::HandlerFields<#handler_fields_type>
        };
    }

    let handler_initializers: Vec<_> = trackable_types
        .iter()
        .rev()
        .map(|field_ty| {
            quote! {
                let fields = <#field_ty as #trackable_trait>::handler_fields(
                    std::rc::Rc::clone(&world),
                    entity,
                    animator.clone(),
                    fields,
                );
            }
        })
        .collect();

    let expanded = quote! {
        use crate::core::{
            SceneWorld as #scene_world_type,
            AnimatorHandle as #animator_handle_type,
            Trackable as #trackable_trait,
            TrackProperty as #track_property_type,
            TrackValueType as #track_value_type_trait,
            Tween as #tween_type,
            components::Draw as #draw_type,
            types::Vector2 as #vector_type,
            TrackableInfo as #trackable_info_type,
            components::Inspection as #inspection_type,
            components::Name as #name_type,
            objects::{
                HandlerRoot as #handler_root_type,
                Object as #object_trait,
                ObjectBuilderComponent as #builder_component_trait,
                ObjectHandler as #object_handler_trait,
                ObjectTrackable as #object_trackable_trait,
            },
        };

        /// Builder generated for this scene object.
        #visibility struct #builder_name {
            object: #object_name,
            name: std::string::String,
        }

        impl #builder_name {
            fn new() -> Self {
                Self {
                    object: <#object_name as Default>::default(),
                    name: stringify!(#object_name).to_owned(),
                }
            }

            /// Sets the user-facing name attached to this object.
            pub fn name(mut self, name: impl Into<std::string::String>) -> Self {
                self.name = name.into();
                self
            }

            /// Spawns the configured object as inactive in `scene` and returns its handler.
            pub fn build(self, scene: &mut crate::core::Scene) -> #handler_name {
                scene.spawn_object::<#object_name>(self.object, self.name)
            }
        }

        #(#component_accessors)*

        /// Typed handler for an entity spawned into a scene.
        #visibility struct #handler_name {
            world: #scene_world_type,
            entity: hecs::Entity,
            animator: #animator_handle_type,
            fields: #handler_fields_type,
        }

        impl std::ops::Deref for #handler_name {
            type Target = #handler_fields_type;

            fn deref(&self) -> &Self::Target {
                &self.fields
            }
        }

        impl #object_handler_trait for #handler_name {
            type Object = #object_name;

            fn get_id(&self) -> hecs::Entity {
                self.entity
            }

            fn get_name(&self) -> std::string::String {
                self.world
                    .borrow()
                    .get::<&#name_type>(self.entity)
                    .expect("Object handler must contain a Name component.")
                    .get()
                    .to_owned()
            }

            fn set_name(&self, name: impl Into<std::string::String>) {
                self.world
                    .borrow()
                    .get::<&mut #name_type>(self.entity)
                    .expect("Object handler must contain a Name component.")
                    .set(name);
            }

            fn get_box(&self) -> #vector_type {
                let world = self.world.borrow();
                let draw = world
                    .get::<&#draw_type>(self.entity)
                    .expect("Object handler must contain a Draw component.");
                (draw.get_box)(&world, self.entity)
            }

            fn get<T: #track_value_type_trait>(
                &self,
                property: #track_property_type<T>,
            ) -> T {
                property
                    .handle(std::rc::Rc::clone(&self.world), self.entity, self.animator.clone())
                    .get()
            }

            fn animate<T: #track_value_type_trait>(
                &self,
                property: #track_property_type<T>,
                to: T,
            ) -> #tween_type<#object_name> {
                property
                    .handle(std::rc::Rc::clone(&self.world), self.entity, self.animator.clone())
                    .animate::<#object_name>(to)
            }

            fn animate_from<T: #track_value_type_trait>(
                &self,
                property: #track_property_type<T>,
                from: T,
                to: T,
            ) -> #tween_type<#object_name> {
                property
                    .handle(std::rc::Rc::clone(&self.world), self.entity, self.animator.clone())
                    .animate_from::<#object_name>(from, to)
            }
        }

        impl #handler_name {
            /// Creates an identical object in the supplied scene.
            pub fn copy(&self, s: &mut crate::core::Scene) -> #handler_name {
                let (object, name) = {
                    let world = self.world.borrow();
                    let object = #object_name {
                        #(#field_idents: (*world
                            .get::<&#field_types>(self.entity)
                            .expect("Object handler must contain its object fields.")).clone(),)*
                    };

                    (object, self.get_name())
                };

                #builder_name {
                    object,
                    name,
                }
                .build(s)
            }
        }

        #(
            impl #object_trackable_trait<#trackable_types> for #object_name {}
        )*

        #[allow(non_upper_case_globals)]
        const #infos_ident: [#trackable_info_type; #count] = [
            #(#trackable_infos),*
        ];

        fn #infos_fn_ident() -> &'static [#trackable_info_type] {
            &#infos_ident
        }

        impl #object_name {
            /// Builds a typed handler from a spawned entity id.
            pub fn handler(world: #scene_world_type, entity: hecs::Entity, animator: #animator_handle_type) -> #handler_name {
                let fields = #handler_root_type::<#object_name>::new();
                #(#handler_initializers)*

                #handler_name { world, entity, animator, fields }
            }
        }

        impl #object_trait for #object_name {
            type Builder = #builder_name;
            type Handler = #handler_name;

            fn builder() -> Self::Builder {
                #builder_name::new()
            }

            fn handler(world: #scene_world_type, entity: hecs::Entity, animator: #animator_handle_type) -> Self::Handler {
                #object_name::handler(world, entity, animator)
            }

            fn inspection() -> #inspection_type {
                #inspection_type {
                    object_name: stringify!(#object_name),
                    get: |_world, _entity| #infos_fn_ident(),
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
