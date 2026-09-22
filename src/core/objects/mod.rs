pub(crate) mod appearance;
mod canvas;
mod creation;
mod image;
mod image_shader;
mod latex_geometry;
mod mesh_shader;
mod node;
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
pub(crate) use creation::{CreationDraw, particle_count_for_bounds, particle_visual_key};
pub(crate) use creation::{MorphParticleRoute, ParticleBatch, silhouette_grid};
pub(crate) use image::ImageSource;
pub use image_shader::{ImageShader, ImageShaderData, ImageShaderUniform};
pub(crate) use image_shader::{
    set_shader_uniform, shader_uniform, shader_uniforms, validate_shader_track_value,
    validate_shader_values,
};
pub(crate) use mesh_shader::{EffectiveMeshShader, effective_mesh_shader};
pub use mesh_shader::{MeshShader, MeshShaderData, MeshShaderUniform};
pub use node::*;
pub use object::*;
pub(crate) use regular_polygon::regular_polygon_vertices;
pub use render::object_box;
pub use simulation::*;
pub use three_d::*;
pub use two_d::*;

#[cfg(test)]
pub(crate) use render::draw_canvas2d;
pub(crate) use render::draw_canvas2d_editor_with_images;
pub(crate) use render::{
    ImageShaderImage, camera_matrix2d, camera_outline_points2d, capture_appearance,
    children_by_z_index, draw_canvas2d_with_shader_images, draw_entity, draw_image_shader_source,
    image_shader_bounds, object_follows_camera,
};
#[cfg(test)]
pub(crate) use render::{outline_points, outline_segments3d, pick_canvas2d, pick_canvas3d};
pub(crate) use render::{
    outline_points_in_world, outline_segments3d_in_world, pick_canvas2d_in_world,
    pick_canvas3d_with_camera,
};
