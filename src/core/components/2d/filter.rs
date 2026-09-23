use kinematic_macros::Trackable;

/// Image filters applied to a two-dimensional object and its subtree.
#[derive(Clone, Default, Trackable)]
pub struct Filter {
    /// Gaussian blur sigma in local canvas units.
    #[track(min = 0.0)]
    pub blur: f32,
}
