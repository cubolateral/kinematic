use crate::core::{
    components::{Draw3D, Transform3D},
    types::Color,
};
use kinematic_macros::{Object, Trackable};

/// Color and intensity shared by three-dimensional lights.
#[derive(Clone, Trackable)]
pub struct Light3D {
    #[track]
    pub color: Color,
    #[track(min = 0.0)]
    pub intensity: f32,
}

impl Default for Light3D {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
}

/// Light shining along its local negative Z axis.
#[derive(Default, Object)]
#[object(spatial = "3d", builder = "directional_light")]
pub struct DirectionalLight3D {
    #[trackable]
    pub light: Light3D,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,

    pub kind: DirectionalLightKind,
}

#[derive(Clone, Default)]
pub struct DirectionalLightKind;

/// Light shining in every direction from its position.
#[derive(Default, Object)]
#[object(spatial = "3d", builder = "point_light")]
pub struct PointLight3D {
    #[trackable]
    pub light: Light3D,
    #[trackable]
    pub attenuation: LightAttenuation,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,

    pub kind: PointLightKind,
}

#[derive(Clone, Default)]
pub struct PointLightKind;

/// Light shining in a cone along its local negative Z axis.
#[derive(Default, Object)]
#[object(spatial = "3d", builder = "spot_light")]
pub struct SpotLight3D {
    #[trackable]
    pub light: Light3D,
    #[trackable]
    pub attenuation: LightAttenuation,
    #[trackable]
    pub cone: SpotLightCone,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,

    pub kind: SpotLightKind,
}

#[derive(Clone, Default)]
pub struct SpotLightKind;

/// Constant, linear, and quadratic falloff coefficients.
#[derive(Clone, Trackable)]
pub struct LightAttenuation {
    #[track(min = 0.0)]
    pub constant: f32,
    #[track(min = 0.0)]
    pub linear: f32,
    #[track(min = 0.0)]
    pub quadratic: f32,
}

impl Default for LightAttenuation {
    fn default() -> Self {
        Self {
            constant: 1.0,
            linear: 0.0,
            quadratic: 0.0,
        }
    }
}

/// Spotlight cone half-angle in radians.
#[derive(Clone, Trackable)]
pub struct SpotLightCone {
    #[track(min = 0.001, max = 3.14159)]
    pub cutoff: f32,
}

impl Default for SpotLightCone {
    fn default() -> Self {
        Self {
            cutoff: std::f32::consts::FRAC_PI_4,
        }
    }
}
