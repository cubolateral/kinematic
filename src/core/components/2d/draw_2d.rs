use crate::core::types::Vector2;
use kinematic_macros::Trackable;

fn centered_box_bounds(world: &hecs::World, entity: hecs::Entity) -> skia_safe::Rect {
    let draw = world.get::<&Draw2D>(entity).unwrap();
    let size = (draw.box_size)(world, entity);
    skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y)
}

/// Local two-dimensional rendering settings and callbacks for an entity.
///
/// The callback receives the current entity state and must not mutate the ECS world.
#[derive(Clone, Trackable)]
pub struct Draw2D {
    /// Whether this entity and, for containers, its subtree are drawn.
    #[track]
    pub visibility: bool,
    /// Transparency applied while drawing this entity, from `0.0` to `1.0`.
    #[track(min = 0.0, max = 1.0)]
    pub opacity: f32,
    /// Stacking order among sibling objects. Higher values are drawn in front.
    #[track]
    pub z_index: i32,
    /// Whether this entity and its subtree are affected by the canvas camera.
    #[track]
    pub follows_camera: bool,

    /// Draws this entity in local coordinates with the supplied opacity.
    pub on_draw: fn(&hecs::World, hecs::Entity, &skia_safe::Canvas, f32),

    /// Returns the object's logical local bounding-box size.
    pub box_size: fn(&hecs::World, hecs::Entity) -> Vector2,

    /// Returns the object's local visual bounds, including displaced ink and outlines.
    pub visual_bounds: fn(&hecs::World, hecs::Entity) -> skia_safe::Rect,
}

impl Default for Draw2D {
    fn default() -> Self {
        Self {
            visibility: true,
            opacity: 1.0,
            z_index: 0,
            follows_camera: true,
            on_draw: |_, _, _, _| {},
            box_size: |_, _| Vector2::ZERO,
            visual_bounds: centered_box_bounds,
        }
    }
}
