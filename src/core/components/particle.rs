use kinematic_macros::Trackable;

pub(crate) const PARTICLE_DISTANCE: f32 = 64.0;
pub(crate) const PARTICLE_RADIUS: f32 = 1.0;
pub(crate) const PARTICLE_STAGGER: f32 = 0.75;
pub(crate) const PARTICLE_FADE_START: f32 = 0.9;

/// Internal state used by particle-based transitions.
///
/// This component is attached to every object by [`crate::core::objects::Object::spawn`], but is not
/// part of an object's inspection metadata. Its tracks are operational effect
/// state rather than authorable object properties.
#[derive(Clone, Default, Trackable, Debug)]
pub(crate) struct Morph {
    /// Normalized transition progress.
    #[track]
    pub progress: f32,
    /// Whether the signature particle cloud is active.
    #[track]
    pub particles_enabled: bool,
}
