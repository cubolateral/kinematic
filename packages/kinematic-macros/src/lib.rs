mod object;
mod scene;
mod trackable;

use proc_macro::TokenStream;

#[proc_macro_derive(Object, attributes(trackable))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    object::derive_object(input)
}

#[proc_macro_derive(Trackable, attributes(track))]
pub fn derive_trackable(input: TokenStream) -> TokenStream {
    trackable::derive_trackable(input)
}

/// Turns a scene-building function into a [`SceneBuilder`](kinematic::core::SceneBuilder) factory.
#[proc_macro_attribute]
pub fn scene(attribute: TokenStream, input: TokenStream) -> TokenStream {
    scene::scene(attribute, input)
}
