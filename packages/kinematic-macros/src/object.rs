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
    let object_trait = format_ident!("__Kinematic{}Object", object_name);
    let object_handler_trait = format_ident!("__Kinematic{}ObjectHandler", object_name);
    let scene_world_type = format_ident!("__Kinematic{}SceneWorld", object_name);
    let trackable_info_type = format_ident!("__Kinematic{}TrackableInfo", object_name);
    let trackable_trait = format_ident!("__Kinematic{}Trackable", object_name);

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
    let mut handler_fields_type = quote!(());

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
                    fields,
                );
            }
        })
        .collect();

    let expanded = quote! {
        use crate::core::{
            SceneWorld as #scene_world_type,
            Trackable as #trackable_trait,
            TrackableInfo as #trackable_info_type,
            components::Inspection as #inspection_type,
            objects::{
                Object as #object_trait,
                ObjectBuilderComponent as #builder_component_trait,
                ObjectHandler as #object_handler_trait,
            },
        };

        /// Builder generated for this scene object.
        #visibility struct #builder_name {
            object: #object_name,
        }

        impl #builder_name {
            /// Creates a builder initialized with the object's defaults.
            pub fn new() -> Self {
                Self {
                    object: <#object_name as Default>::default(),
                }
            }

            /// Finishes configuring the scene object.
            pub fn build(self) -> #object_name {
                self.object
            }
        }

        impl Default for #builder_name {
            fn default() -> Self {
                Self::new()
            }
        }

        #(#component_accessors)*

        /// Typed handler for an entity spawned into a scene.
        #visibility struct #handler_name {
            entity: hecs::Entity,
            fields: #handler_fields_type,
        }

        impl std::ops::Deref for #handler_name {
            type Target = #handler_fields_type;

            fn deref(&self) -> &Self::Target {
                &self.fields
            }
        }

        impl #object_handler_trait for #handler_name {
            fn entity(&self) -> hecs::Entity {
                self.entity
            }
        }

        #[allow(non_upper_case_globals)]
        const #infos_ident: [#trackable_info_type; #count] = [
            #(#trackable_infos),*
        ];

        fn #infos_fn_ident() -> &'static [#trackable_info_type] {
            &#infos_ident
        }

        impl #object_name {
            /// Builds a typed handler from a spawned entity id.
            pub fn handler(world: #scene_world_type, entity: hecs::Entity) -> #handler_name {
                let fields = ();
                #(#handler_initializers)*

                #handler_name { entity, fields }
            }
        }

        impl #object_trait for #object_name {
            type Handler = #handler_name;

            fn handler(world: #scene_world_type, entity: hecs::Entity) -> Self::Handler {
                #object_name::handler(world, entity)
            }

            fn inspection() -> #inspection_type {
                #inspection_type {
                    get: |_world, _entity| #infos_fn_ident(),
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
