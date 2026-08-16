use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

/// Maps a field's Rust type to its `TrackValue` variant name.
/// Add a new arm here whenever a new `TrackValue` variant is added.
fn track_value_variant(ty: &Type) -> proc_macro2::Ident {
    let ty_str = quote!(#ty).to_string();
    match ty_str.as_str() {
        "f32" => format_ident!("F32"),
        // "String" => format_ident!("String"),
        other => panic!(
            "Trackable: no TrackValue variant mapped for type `{other}` — \
             add a `TrackValue` variant and a match arm in `track_value_variant`"
        ),
    }
}

// Generates the per-field handle type for a `Trackable` component.
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
        .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("track")))
        .collect();

    let count = tracked_fields.len();
    let mut track_entries = Vec::with_capacity(count);
    let mut tween_fns = Vec::with_capacity(count);

    for (id, field) in tracked_fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let id = id as u32;
        let field_name = field_ident.to_string();
        let variant = track_value_variant(field_ty);

        track_entries.push(quote! {
            crate::core::TrackInfo {
                id: #id,
                name: #field_name,
                get: |world, entity| {
                    crate::core::TrackValue::#variant(
                        world.get::<&#struct_name>(entity).unwrap().#field_ident.clone()
                    )
                },
                set: |world, entity, value| {
                    if let crate::core::TrackValue::#variant(value) = value {
                        world.get::<&mut #struct_name>(entity).unwrap().#field_ident = value;
                    }
                },
            }
        });

        tween_fns.push(quote! {
            pub fn #field_ident(self, value: #field_ty) -> crate::core::Tween {
                let old_value = {
                    let mut component = self
                        .scene
                        .get_world_mut()
                        .get::<&mut #struct_name>(self.entity)
                        .unwrap();
                    let old_value = component.#field_ident.clone();
                    component.#field_ident = value.clone();
                    old_value
                };

                crate::core::Tween::new(
                    self.entity,
                    std::any::TypeId::of::<#struct_name>(),
                    #id,
                    <#struct_name as crate::core::Trackable>::track(#id).set,
                    crate::core::TrackValue::#variant(old_value),
                    crate::core::TrackValue::#variant(value),
                )
            }
        });
    }

    let tracks_ident = format_ident!("__{}_TRACKS", struct_name.to_string().to_uppercase());
    let expanded = quote! {
        /// Typed access wrapper around an entity's tracked component.
        pub struct #handle_name<'a> {
            /// Scene world borrow needed to read and write the component.
            scene: &'a mut crate::core::Scene,
            /// Entity that owns the tracked component.
            entity: hecs::Entity,
        }

        impl #struct_name {
            /// Builds the generated handle for this trackable component.
            pub fn handle<'a>(
                scene: &'a mut crate::core::Scene,
                entity: hecs::Entity,
            ) -> #handle_name<'a> {
                #handle_name { scene, entity }
            }
        }

        impl<'a> #handle_name<'a> {
            #(#tween_fns)*
        }

        #[allow(non_upper_case_globals)]
        const #tracks_ident: [crate::core::TrackInfo; #count] = [
            #(#track_entries),*
        ];

        impl crate::core::Trackable for #struct_name {
            type Handle<'a>
                = #handle_name<'a>
            where
                Self: 'a;

            fn handle<'a>(
                scene: &'a mut crate::core::Scene,
                entity: hecs::Entity,
            ) -> Self::Handle<'a> {
                #handle_name { scene, entity }
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
