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

/// Generates the typed handle and track metadata for a `Trackable` component.
pub fn derive_trackable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let handle_name = format_ident!("{}Handle", struct_name);

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
                T: crate::core::objects::ObjectBuilderComponent<#struct_name>,
            {
                fn #field_ident(mut self, value: #field_ty) -> Self {
                    <T as crate::core::objects::ObjectBuilderComponent<#struct_name>>::component_mut(
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
                fn assert_track_value_type<T: crate::core::TrackValueType>() {}
                assert_track_value_type::<#field_ty>
            };
        });

        track_entries.push(quote! {
            crate::core::TrackInfo {
                id: #id,
                name: #field_name,
                get: |world, entity| {
                    <#field_ty as crate::core::TrackValueType>::into_track_value(
                        world.get::<&#struct_name>(entity).unwrap().#field_ident.clone()
                    )
                },
                set: |world, entity, value| {
                    if let Some(value) = <#field_ty as crate::core::TrackValueType>::from_track_value(value) {
                        world.get::<&mut #struct_name>(entity).unwrap().#field_ident = value;
                    }
                },
            }
        });

        handle_fields.push(quote! {
            #field_visibility #field_ident: crate::core::TrackHandle<#field_ty>,
        });

        handle_initializers.push(quote! {
            #field_ident: crate::core::TrackHandle::new(
                std::rc::Rc::clone(&world),
                entity,
                std::any::TypeId::of::<#struct_name>(),
                <#struct_name as crate::core::Trackable>::track(#id),
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
                self,
                value: <#field_ty as crate::core::TrackValueType>::Input,
            ) -> crate::core::Tween {
                self.#field_ident.set(value.into())
            }
        });
    }

    let tracks_ident = format_ident!("__{}_TRACKS", struct_name.to_string().to_uppercase());
    let expanded = quote! {
        #(#type_assertions)*
        #(#builder_setters)*

        /// Typed access wrapper around an entity's tracked component.
        pub struct #handle_name {
            #(#handle_fields)*
        }

        impl #struct_name {
            /// Builds the generated handle for this trackable component.
            pub fn handle(
                world: crate::core::SceneWorld,
                entity: hecs::Entity,
            ) -> #handle_name {
                #handle_name {
                    #(#handle_initializers)*
                }
            }
        }

        impl #handle_name {
            #(#tween_fns)*
        }

        #[allow(non_upper_case_globals)]
        const #tracks_ident: [crate::core::TrackInfo; #count] = [
            #(#track_entries),*
        ];

        impl crate::core::Trackable for #struct_name {
            type Handle = #handle_name;

            fn handle(
                world: crate::core::SceneWorld,
                entity: hecs::Entity,
            ) -> Self::Handle {
                #struct_name::handle(world, entity)
            }

            fn track(id: crate::core::TrackId) -> &'static crate::core::TrackInfo {
                &#tracks_ident[id as usize]
            }

            fn info() -> &'static crate::core::TrackableInfo {
                &Self::INFO
            }
        }

        impl #struct_name {
            pub const INFO: crate::core::TrackableInfo = crate::core::TrackableInfo {
                name: stringify!(#struct_name),
                get: || &#tracks_ident,
            };
        }
    };

    proc_macro::TokenStream::from(expanded)
}
