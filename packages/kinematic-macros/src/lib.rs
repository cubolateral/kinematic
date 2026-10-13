mod container;
mod object;
mod track_enum;
mod trackable;

use proc_macro::TokenStream;

fn documentation(attributes: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let lines = attributes.iter().filter_map(|attribute| {
        if !attribute.path().is_ident("doc") {
            return None;
        }
        let syn::Meta::NameValue(value) = &attribute.meta else {
            return None;
        };
        Some(&value.value)
    });
    quote::quote!(concat!(#(#lines, "\n",)*))
}

#[proc_macro_derive(Object, attributes(trackable, object))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    object::derive_object(input)
}

/// Marks an object handler as able to own child objects.
#[proc_macro_derive(Node)]
pub fn derive_node(input: TokenStream) -> TokenStream {
    container::derive_node(input)
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

mod lua_methods;

/// Binds explicitly annotated methods from one connected handler impl.
#[proc_macro_attribute]
pub fn lua_methods(_args: TokenStream, input: TokenStream) -> TokenStream {
    lua_methods::expand(input)
}
