use kinematic_macros::Trackable;

pub(crate) const PARTICLE_COUNT: u32 = 4096;
pub(crate) const PARTICLE_DISTANCE: f32 = 32.0;
pub(crate) const PARTICLE_RADIUS: f32 = 1.0;
pub(crate) const PARTICLE_STAGGER: f32 = 0.25;
pub(crate) const PARTICLE_FADE_START: f32 = 0.7;

/// Particle configuration used by the signature creation effect.
#[derive(Clone, Trackable, Debug)]
pub struct ParticleStyle {
    /// Whether particles control the current style progress.
    #[track]
    pub particles_enabled: bool,
}

impl Default for ParticleStyle {
    fn default() -> Self {
        Self {
            particles_enabled: false,
        }
    }
}
