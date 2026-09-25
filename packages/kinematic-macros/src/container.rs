use quote::format_ident;
use syn::{DeriveInput, parse_macro_input};

pub fn derive_node(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let object_name = &input.ident;
    let handler_name = format_ident!("{}Handler", object_name);

    quote::quote! {
        impl kinematic::core::objects::Node for #object_name {}

        impl #handler_name {
            /// Adds an object subtree to this container at the current scheduling time.
            pub fn add(&self, handler: &impl kinematic::core::objects::ObjectHandler) {
                <Self as kinematic::core::objects::NodeHandler>::add(self, handler);
            }

            /// Inserts an object subtree at `index` at the current scheduling time.
            pub fn insert(
                &self,
                handler: &impl kinematic::core::objects::ObjectHandler,
                index: usize,
            ) {
                <Self as kinematic::core::objects::NodeHandler>::insert(self, handler, index);
            }

            /// Ends the lifetime of every direct child at the current scheduling time.
            pub fn remove_children(&self) {
                <Self as kinematic::core::objects::NodeHandler>::remove_children(self);
            }

            /// Returns the entity ids of all direct children in insertion order.
            pub fn children(&self) -> Vec<hecs::Entity> {
                <Self as kinematic::core::objects::NodeHandler>::children(self)
            }

            /// Returns the entity id of the direct child at `index`.
            pub fn get_child_entity(
                &self,
                index: usize,
            ) -> Result<hecs::Entity, kinematic::core::objects::ChildError> {
                <Self as kinematic::core::objects::NodeHandler>::get_child_entity(self, index)
            }

            /// Returns the typed handler for a direct child at `index`.
            pub fn get_child<T: kinematic::core::objects::Object + 'static>(
                &self,
                index: usize,
            ) -> Result<T::Handler, kinematic::core::objects::ChildError> {
                <Self as kinematic::core::objects::NodeHandler>::get_child::<T>(self, index)
            }

            /// Returns the first direct child with object type `T`.
            pub fn first_child<T: kinematic::core::objects::Object + 'static>(
                &self,
            ) -> Option<T::Handler> {
                <Self as kinematic::core::objects::NodeHandler>::first_child::<T>(self)
            }

            /// Returns the last direct child with object type `T`.
            pub fn last_child<T: kinematic::core::objects::Object + 'static>(
                &self,
            ) -> Option<T::Handler> {
                <Self as kinematic::core::objects::NodeHandler>::last_child::<T>(self)
            }

            /// Returns all direct children with object type `T` in insertion order.
            pub fn all_children<T: kinematic::core::objects::Object + 'static>(
                &self,
            ) -> Vec<T::Handler> {
                <Self as kinematic::core::objects::NodeHandler>::all_children::<T>(self)
            }

            /// Returns the first descendant with object type `T` in depth-first order.
            pub fn first_child_recursively<
                T: kinematic::core::objects::Object + 'static,
            >(&self) -> Option<T::Handler> {
                <Self as kinematic::core::objects::NodeHandler>::first_child_recursively::<T>(self)
            }

            /// Returns the last descendant with object type `T` in depth-first order.
            pub fn last_child_recursively<
                T: kinematic::core::objects::Object + 'static,
            >(&self) -> Option<T::Handler> {
                <Self as kinematic::core::objects::NodeHandler>::last_child_recursively::<T>(self)
            }

            /// Returns all descendants with object type `T` in depth-first order.
            pub fn all_children_recursively<
                T: kinematic::core::objects::Object + 'static,
            >(&self) -> Vec<T::Handler> {
                <Self as kinematic::core::objects::NodeHandler>::all_children_recursively::<T>(self)
            }
        }

        impl kinematic::core::objects::NodeHandler for #handler_name {
            fn container_world(&self) -> kinematic::core::SceneWorld {
                std::rc::Rc::clone(&self.world)
            }

            fn container_entity(&self) -> hecs::Entity {
                self.entity
            }

            fn container_time(&self) -> f32 {
                self.animator.assert_finite_scope();
                self.animator.time()
            }

            fn container_animator(&self) -> kinematic::core::AnimatorHandle {
                self.animator.clone()
            }
        }
    }
    .into()
}
