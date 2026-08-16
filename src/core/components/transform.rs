use kinematic_macros::Trackable;

#[derive(Trackable, Default, Debug)]
/// Position of an entity in logical canvas coordinates.
pub struct Transform {
    #[track]
    pub x: f32,
    #[track]
    pub y: f32,
}
