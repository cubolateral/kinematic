mod color;
mod quad;
mod rect;
mod vector2;

pub use color::*;
pub use quad::*;
pub use rect::*;
pub use vector2::*;

/// Three-dimensional vector used by scene geometry and transforms.
pub type Vector3 = glam::Vec3;
/// Unit quaternion used by three-dimensional rotations.
pub type Quaternion = glam::Quat;
/// Creates a three-dimensional vector.
pub const fn vec3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3::new(x, y, z)
}
