use crate::core::{
    SceneWorld,
    components::{Camera2D, Camera3D},
    objects::{Canvas2DHandler, Canvas3DHandler, ObjectHandler},
    types::Color,
};
use kinematic_macros::Trackable;

/// Logical canvas output, independent of graphics resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasTexture {
    pub(crate) scene: u64,
    pub(crate) entity: hecs::Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CanvasDimension {
    Two,
    Three,
}

/// Declarative render target settings with a build-time resolution.
#[derive(Clone, Trackable)]
pub struct CanvasSettings {
    #[track]
    pub clear: Color,

    pub(crate) resolution: (u32, u32),
    pub(crate) dimension: CanvasDimension,
}

impl CanvasSettings {
    pub(crate) fn new(dimension: CanvasDimension) -> Self {
        Self {
            clear: Color::TRANSPARENT,
            resolution: (0, 0),
            dimension,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        let (w, h) = self.resolution;
        if w == 0 || h == 0 || w > i32::MAX as u32 || h > i32::MAX as u32 {
            return Err(
                "Canvas resolution must be explicitly set to positive i32 dimensions.".into(),
            );
        }
        Ok(())
    }
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }
    pub fn aspect_ratio(&self) -> Result<f32, String> {
        self.validate()?;
        Ok(self.resolution.0 as f32 / self.resolution.1 as f32)
    }
}

/// Canvas reference used by a projection, without a GPU texture in the scene.
#[derive(Clone, Default)]
pub struct ProjectionSource(pub(crate) Option<CanvasTexture>);

mod sealed {
    pub trait Sealed {}
}

/// Canvas handler accepted as the source of a 2D or 3D projection.
#[doc(hidden)]
pub trait ProjectionCanvas: ObjectHandler + sealed::Sealed {
    fn projection_texture(&self) -> CanvasTexture;

    fn projection_resolution(&self) -> (u32, u32) {
        self.object_world()
            .borrow()
            .get::<&CanvasSettings>(self.get_id())
            .expect("Canvas handler must contain CanvasSettings.")
            .resolution
    }
}

impl sealed::Sealed for Canvas2DHandler {}
impl ProjectionCanvas for Canvas2DHandler {
    fn projection_texture(&self) -> CanvasTexture {
        self.get_texture()
    }
}

impl sealed::Sealed for Canvas3DHandler {}
impl ProjectionCanvas for Canvas3DHandler {
    fn projection_texture(&self) -> CanvasTexture {
        self.get_texture()
    }
}

pub(crate) fn scene_identity(world: &SceneWorld) -> u64 {
    world
        .borrow()
        .query::<&crate::core::SceneIdentity>()
        .iter()
        .next()
        .unwrap()
        .0
}

pub(crate) fn validate_canvas(world: &hecs::World, entity: hecs::Entity) -> Result<(), String> {
    let settings = world
        .get::<&CanvasSettings>(entity)
        .map_err(|_| "Canvas is unavailable.")?;
    settings.validate()?;
    match settings.dimension {
        CanvasDimension::Two => world
            .get::<&Camera2D>(entity)
            .map_err(|_| "Canvas2D camera is missing.")?
            .validate()?,
        CanvasDimension::Three => world
            .get::<&Camera3D>(entity)
            .map_err(|_| "Canvas3D camera is missing.")?
            .validate()?,
    }
    Ok(())
}
