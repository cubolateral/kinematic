use crate::core::{
    SceneWorld,
    components::Node,
    objects::{
        CameraTransform2D, Canvas2DHandler, Canvas3DHandler, ObjectHandler, Perspective,
        contains_entity,
    },
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
    pub(crate) camera: Option<hecs::Entity>,
    pub(crate) dimension: CanvasDimension,
}

impl CanvasSettings {
    pub(crate) fn new(dimension: CanvasDimension) -> Self {
        Self {
            clear: Color::TRANSPARENT,
            resolution: (0, 0),
            camera: None,
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
    let Some(camera) = settings.camera else {
        return if settings.dimension == CanvasDimension::Two {
            Ok(())
        } else {
            Err("Canvas3D requires an explicitly assigned Camera3D.".into())
        };
    };
    if !contains_entity(world, entity, camera) {
        return Err("Assigned camera must belong to its canvas subtree.".into());
    }
    let mut ancestor = Some(camera);
    while let Some(current) = ancestor {
        if !world
            .get::<&Node>(current)
            .is_ok_and(|node| node.is_activated)
        {
            return Err("Assigned camera is inactive.".into());
        }
        if current == entity {
            break;
        }
        ancestor = world.get::<&Node>(current).ok().and_then(|n| n.parent);
    }
    match settings.dimension {
        CanvasDimension::Two => {
            let camera = world
                .get::<&CameraTransform2D>(camera)
                .map_err(|_| "Canvas2D requires a Camera.")?;
            if !camera.position.is_finite()
                || !camera.rotation.is_finite()
                || !camera.zoom.is_finite()
                || camera.zoom <= 0.0
            {
                return Err("Canvas2D camera requires finite values and positive zoom.".into());
            }
        }
        CanvasDimension::Three => {
            world
                .get::<&Perspective>(camera)
                .map_err(|_| "Canvas3D requires a Camera3D.")?
                .validate()?;
        }
    }
    Ok(())
}
