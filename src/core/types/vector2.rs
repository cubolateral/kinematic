/// Two-dimensional vector used by scene geometry and transforms.
pub type Vector2 = glam::Vec2;

/// Creates a two-dimensional vector from horizontal and vertical components.
///
/// This is a shortcut to `Vector2::new(x, y)`.
pub const fn vec2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}
