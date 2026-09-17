mod container;
mod object;
mod scene;
mod track_enum;
mod trackable;

use proc_macro::TokenStream;

#[proc_macro_derive(Object, attributes(trackable, object))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    object::derive_object(input)
}

/// Marks an object handler as able to own child objects.
#[proc_macro_derive(Container)]
pub fn derive_container(input: TokenStream) -> TokenStream {
    container::derive_container(input)
}

#[proc_macro_derive(Trackable, attributes(track))]
pub fn derive_trackable(input: TokenStream) -> TokenStream {
    trackable::derive_trackable(input)
}

/// Makes a fieldless enum usable as a discrete animation track value.
#[proc_macro_derive(TrackEnum)]
pub fn derive_track_enum(input: TokenStream) -> TokenStream {
    track_enum::derive_track_enum(input)
}

/// Turns a scene-building function into a Kinematic scene factory.
#[proc_macro_attribute]
pub fn scene(attribute: TokenStream, input: TokenStream) -> TokenStream {
    scene::scene(attribute, input)
}
