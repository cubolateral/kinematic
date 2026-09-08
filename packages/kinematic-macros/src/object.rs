use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

pub fn derive_object(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let object_name = &input.ident;
    let visibility = &input.vis;
    let builder_name = format_ident!("{}Builder", object_name);
    let handler_name = format_ident!("{}Handler", object_name);
    let mut alias = None;
    let mut spatial = None;
    let mut morph = false;
    for attr in &input.attrs {
        if attr.path().is_ident("morph") {
            morph = true;
        } else if attr.path().is_ident("object") {
            if let Err(error) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("builder") {
                    alias = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                } else if meta.path.is_ident("spatial") {
                    spatial = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                } else {
                    return Err(meta.error("Unknown object option."));
                }
                Ok(())
            }) {
                return error.to_compile_error().into();
            }
        }
    }
    let alias = match alias {
        Some(alias) => alias,
        None => {
            return syn::Error::new_spanned(
                object_name,
                "Object must declare an explicit builder name with `builder = \"...\"`.",
            )
            .to_compile_error()
            .into();
        }
    };
    let spatial = match spatial {
        Some(spatial) => spatial,
        None => {
            return syn::Error::new_spanned(
                object_name,
                "Object must declare an explicit spatial type: `2d`, `3d` or `none`.",
            )
            .to_compile_error()
            .into();
        }
    };
    if !["2d", "3d", "none"].contains(&spatial.as_str()) {
        return syn::Error::new_spanned(object_name, "Spatial dimension must be 2d, 3d or none.")
            .to_compile_error()
            .into();
    }
    if morph && spatial != "2d" {
        return syn::Error::new_spanned(object_name, "Morph requires a 2D object.")
            .to_compile_error()
            .into();
    }
    let builder_alias = match syn::parse_str::<syn::Ident>(&alias) {
        Ok(alias) => alias,
        Err(_) => {
            return syn::Error::new_spanned(
                object_name,
                "Builder name must be a valid identifier.",
            )
            .to_compile_error()
            .into();
        }
    };
    let builder_component_trait = quote!(kinematic::core::objects::ObjectBuilderComponent);
    let inspection_type = quote!(kinematic::core::components::Inspection);
    let name_type = quote!(kinematic::core::components::Name);
    let object_trait = quote!(kinematic::core::objects::Object);
    let object_handler_trait = quote!(kinematic::core::objects::ObjectHandler);
    let handler_root_type = quote!(kinematic::core::objects::HandlerRoot);
    let object_trackable_trait = quote!(kinematic::core::objects::ObjectTrackable);
    let scene_world_type = quote!(kinematic::core::SceneWorld);
    let animator_handle_type = quote!(kinematic::core::AnimatorHandle);
    let trackable_info_type = quote!(kinematic::core::TrackableInfo);
    let trackable_trait = quote!(kinematic::core::Trackable);
    let track_property_type = quote!(kinematic::core::TrackProperty);
    let track_value_type_trait = quote!(kinematic::core::TrackValueType);
    let tween_type = quote!(kinematic::core::Tween);
    let vector_type = quote!(kinematic::core::types::Vector2);
    let object_box_fn = quote!(kinematic::core::objects::object_box);
    let object_global_position_fn = quote!(kinematic::core::objects::object_global_position);
    let object_global_rotation_fn = quote!(kinematic::core::objects::object_global_rotation);
    let object_global_scale_fn = quote!(kinematic::core::objects::object_global_scale);
    let object_global_opacity_fn = quote!(kinematic::core::objects::object_global_opacity);
    let remove_object_fn = quote!(kinematic::core::objects::remove_object);
    let save_object_fn = quote!(kinematic::core::objects::save_object);
    let restore_object_fn = quote!(kinematic::core::objects::restore_object);

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

    let spatial_impl = match spatial.as_str() {
        "2d" => quote! {
        impl kinematic::core::objects::Object2DHandler for #handler_name {
            fn get_box(&self) -> #vector_type {
                let world = self.world.borrow();
                #object_box_fn(&world, self.entity)
            }

            fn get_global_position(&self) -> #vector_type {
                let world = self.world.borrow();
                #object_global_position_fn(&world, self.entity)
            }

            fn get_global_rotation(&self) -> f32 {
                let world = self.world.borrow();
                #object_global_rotation_fn(&world, self.entity)
            }

            fn get_global_scale(&self) -> #vector_type {
                let world = self.world.borrow();
                #object_global_scale_fn(&world, self.entity)
            }

            fn get_global_opacity(&self) -> f32 {
                let world = self.world.borrow();
                #object_global_opacity_fn(&world, self.entity)
            }

        }

        },
        "3d" => quote! {
            impl kinematic::core::objects::Object3DHandler for #handler_name {}
        },
        _ => quote! {},
    };
    let morph_impl = if morph {
        quote! { impl kinematic::core::objects::Morphable for #object_name {} }
    } else {
        quote! {}
    };

    let expanded = quote! {
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
            pub fn build(self, s: &mut kinematic::core::Scene) -> #handler_name {
                s.spawn_object::<#object_name>(self.object, self.name)
            }
        }

        #[doc = concat!("Creates a builder for [`", stringify!(#object_name), "`].")]
        #visibility fn #builder_alias() -> #builder_name {
            #builder_name::new()
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

            fn object_world(&self) -> #scene_world_type {
                std::rc::Rc::clone(&self.world)
            }

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

            fn remove(&self) {
                #remove_object_fn(&self.world, self.entity, self.animator.time());
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

            fn save(&self) {
                #save_object_fn(&self.world, self.entity);
            }

            fn restore(&self) -> #tween_type<#object_name> {
                #restore_object_fn(
                    &self.world,
                    self.entity,
                    self.animator.clone(),
                )
            }
        }

        #spatial_impl
        #morph_impl

        impl #handler_name {
            /// Creates an identical object in the supplied scene.
            pub fn copy(&self, s: &mut kinematic::core::Scene) -> #handler_name {
                let (object, name) = {
                    let world = self.world.borrow();
                    let object = #object_name {
                        #(#field_idents: (*world
                            .get::<&#field_types>(self.entity)
                            .expect("Object handler must contain its object fields.")).clone(),)*
                    };

                    (object, <Self as #object_handler_trait>::get_name(self))
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
            type Handler = #handler_name;
            const MORPHABLE: bool = #morph;

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
