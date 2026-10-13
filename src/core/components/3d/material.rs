use crate::core::types::Color;
use kinematic_macros::Trackable;

/// Basic metallic-roughness surface appearance and screen-space outline.
#[derive(Clone, Trackable)]
pub struct Material {
    /// Draws the surface without lighting when true.
    #[track]
    pub unlit: bool,
    /// Base surface color used by lighting.
    #[track]
    pub albedo: Color,
    /// Metallic fraction, from zero for a dielectric to one for a metal.
    #[track(min = 0.0, max = 1.0)]
    pub metallic: f32,
    /// Surface roughness, from 0.04 for a smooth surface to one for a rough surface.
    #[track(min = 0.04, max = 1.0)]
    pub roughness: f32,
    /// Color of the mesh outline.
    #[track]
    pub outline_color: Color,
    /// Width of the mesh outline in screen pixels.
    #[track(min = 0.0)]
    pub outline_width: f32,
    /// Surface opacity, from zero for transparent to one for opaque.
    #[track(min = 0.0, max = 1.0)]
    pub opacity: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Color::WHITE,
            opacity: 1.0,
            metallic: 0.0,
            roughness: 0.7,
            unlit: false,
            outline_color: Color::WHITE,
            outline_width: 0.0,
        }
    }
}
