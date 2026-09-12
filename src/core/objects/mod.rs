pub(crate) mod appearance;
mod canvas;
mod container;
mod creation;
mod image;
mod latex_geometry;
mod object;
pub(crate) mod particle;
mod regular_polygon;
mod render;
mod simulation;
mod string_morph;
#[path = "3d/mod.rs"]
mod three_d;
#[path = "2d/mod.rs"]
mod two_d;

pub use canvas::*;
pub use container::*;
pub(crate) use creation::{CreationDraw, particle_count_for_bounds, particle_visual_key};
pub(crate) use creation::{MorphParticleRoute, ParticleBatch, silhouette_grid};
pub(crate) use image::ImageSource;
pub use object::*;
pub(crate) use regular_polygon::regular_polygon_vertices;
pub use render::object_box;
pub use simulation::*;
pub use three_d::*;
pub use two_d::*;

#[cfg(test)]
pub(crate) use render::draw_canvas2d;
pub(crate) use render::draw_canvas2d_with_images;
pub(crate) use render::{camera_matrix2d, children_by_z_index, draw_entity};
pub(crate) use render::{outline_points, outline_segments3d, pick_canvas2d, pick_canvas3d};
