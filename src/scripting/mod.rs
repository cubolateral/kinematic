//! Lua project loading and isolated scene revisions.
mod native;
mod project;
mod runtime;
mod value;

pub use runtime::LuaScene;
mod sources;
pub use sources::Sources;

pub use project::{LuaProject, MANIFEST, Manifest};

mod definitions;
pub use definitions::definitions;

mod snapshot;

mod adapters;
mod drawing;
mod extension;
mod resources;

mod callback;

pub(crate) fn release_unused() {
    resources::release_unused();

    drawing::release_unused();
}

mod animation;
mod methods;

mod code;

mod audio;
