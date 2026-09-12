use crate::core::types::Color;
use kinematic_macros::Trackable;

/// Basic metallic-roughness surface appearance.
#[derive(Clone, Trackable)]
pub struct Material {
    #[track]
    pub albedo: Color,
    #[track(min = 0.0, max = 1.0)]
    pub opacity: f32,
    #[track(min = 0.0, max = 1.0)]
    pub metallic: f32,
    #[track(min = 0.04, max = 1.0)]
    pub roughness: f32,
    #[track]
    pub unlit: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Color::WHITE,
            opacity: 1.0,
            metallic: 0.0,
            roughness: 0.7,
            unlit: false,
        }
    }
}
