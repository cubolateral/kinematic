mod object;
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
