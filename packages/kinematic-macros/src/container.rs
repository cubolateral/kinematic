use quote::format_ident;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_container(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let object_name = &input.ident;
    let handler_name = format_ident!("{}Handler", object_name);

    quote::quote! {
        impl crate::core::objects::Container for #object_name {}

        impl #handler_name {
            /// Adds an object subtree to this container at the current scheduling time.
            pub fn add(&self, handler: &impl crate::core::objects::ObjectHandler) {
                <Self as crate::core::objects::ContainerHandler>::add(self, handler);
            }
        }

        impl crate::core::objects::ContainerHandler for #handler_name {
            fn container_world(&self) -> crate::core::SceneWorld {
                std::rc::Rc::clone(&self.world)
            }

            fn container_entity(&self) -> hecs::Entity {
                self.entity
            }

            fn container_time(&self) -> f32 {
                self.animator.time()
            }
        }
    }
    .into()
}
