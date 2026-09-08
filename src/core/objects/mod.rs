mod canvas;
mod container;
mod creation;
mod latex_geometry;
mod object;
pub(crate) mod particle;
mod render;
mod string_morph;
#[path = "3d/mod.rs"]
mod three_d;
#[path = "2d/mod.rs"]
mod two_d;

pub use canvas::*;
pub use container::*;
pub(crate) use creation::{CreationDraw, particle_visual_key};
pub(crate) use creation::{MorphParticleRoute, draw_particle_batch, silhouette_grid};
pub use object::*;
pub use render::object_box;
pub use three_d::*;
pub use two_d::*;

#[cfg(test)]
pub(crate) use render::draw_canvas2d;
pub(crate) use render::draw_canvas2d_with_images;
pub(crate) use render::{active_camera_matrix, draw_entity};
pub(crate) use render::{draw_canvas_outline2d, pick_canvas2d};
