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

/// Generates object-handler fields and track metadata for a `Trackable` component.
pub fn derive_trackable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let handler_fields_name = format_ident!("__{}HandlerFields", struct_name);
    let builder_component_trait = format_ident!("__Kinematic{}BuilderComponent", struct_name);
    let scene_world_type = format_ident!("__Kinematic{}SceneWorld", struct_name);
    let track_handle_type = format_ident!("__Kinematic{}TrackHandle", struct_name);
    let track_id_type = format_ident!("__Kinematic{}TrackId", struct_name);
    let track_info_type = format_ident!("__Kinematic{}TrackInfo", struct_name);
    let trackable_info_type = format_ident!("__Kinematic{}TrackableInfo", struct_name);
    let trackable_trait = format_ident!("__Kinematic{}Trackable", struct_name);
    let track_value_type_trait = format_ident!("__Kinematic{}TrackValueType", struct_name);
    let tween_type = format_ident!("__Kinematic{}Tween", struct_name);

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
        let field_visibility = &field.vis;
        let id = id as u32;
        let field_name = field_ident.to_string();

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
            #field_visibility #field_ident: #track_handle_type<#field_ty>,
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
            ),
        });

        tween_fns.push(quote! {
            #field_visibility fn #field_ident(
                &self,
                value: <#field_ty as #track_value_type_trait>::Input,
            ) -> #tween_type {
                self.#field_ident.set(value.into())
            }
        });
    }

    let tracks_ident = format_ident!("__{}_TRACKS", struct_name.to_string().to_uppercase());
    let expanded = quote! {
        use crate::core::{
            SceneWorld as #scene_world_type,
            TrackHandle as #track_handle_type,
            TrackId as #track_id_type,
            TrackInfo as #track_info_type,
            Trackable as #trackable_trait,
            TrackableInfo as #trackable_info_type,
            TrackValueType as #track_value_type_trait,
            Tween as #tween_type,
            objects::ObjectBuilderComponent as #builder_component_trait,
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

        impl<Next> #handler_fields_name<Next> {
            #(#tween_fns)*
        }

        #[allow(non_upper_case_globals)]
        const #tracks_ident: [#track_info_type; #count] = [
            #(#track_entries),*
        ];

        impl #trackable_trait for #struct_name {
            type HandlerFields<Next> = #handler_fields_name<Next>;

            fn handler_fields<Next>(
                world: #scene_world_type,
                entity: hecs::Entity,
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
            pub const INFO: #trackable_info_type = #trackable_info_type {
                name: stringify!(#struct_name),
                get: || &#tracks_ident,
            };
        }
    };

    proc_macro::TokenStream::from(expanded)
}
