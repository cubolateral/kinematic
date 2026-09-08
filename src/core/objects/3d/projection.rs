use crate::core::{
    components::{
        Draw3D, GeometryKey, RenderContext3D, Transform3D, render_states, validate_dimensions,
        validate_transformation,
    },
    objects::{Canvas2DHandler, CanvasSettings, ObjectHandler, PlaneShape, global_matrix3d},
};
use kinematic_macros::Object;
use three_d::Geometry;

use super::super::canvas::ProjectionSource;

#[derive(Clone, Copy)]
pub(crate) struct ProjectionSettings {
    pixels_per_unit: f32,
}

impl Default for ProjectionSettings {
    fn default() -> Self {
        Self {
            pixels_per_unit: 256.0,
        }
    }
}

/// Unlit plane sampling the premultiplied output of a Canvas2D.
#[derive(Object, hecs::Bundle)]
#[object(spatial = "3d", builder = "projection")]
pub struct Projection {
    #[trackable]
    pub shape: PlaneShape,
    #[trackable]
    pub transform: Transform3D,
    #[trackable]
    pub draw: Draw3D,
    pub source: ProjectionSource,
    settings: ProjectionSettings,
}

impl Default for Projection {
    fn default() -> Self {
        Self {
            shape: PlaneShape::default(),
            transform: Transform3D::default(),
            draw: Draw3D {
                on_draw: draw_projection,
                get_box: |world, entity| {
                    world
                        .get::<&PlaneShape>(entity)
                        .unwrap()
                        .size
                        .abs()
                        .extend(0.0)
                },
                ..Default::default()
            },
            source: ProjectionSource::default(),
            settings: ProjectionSettings::default(),
        }
    }
}

impl ProjectionBuilder {
    /// Sets the Canvas2D-to-world scale used to size the projection plane.
    pub fn pixels_per_unit(mut self, pixels_per_unit: f32) -> Self {
        assert!(
            pixels_per_unit.is_finite() && pixels_per_unit > 0.0,
            "Projection pixels per unit must be finite and positive."
        );
        if self.object.source.0.is_some() {
            self.object.shape.size *= self.object.settings.pixels_per_unit / pixels_per_unit;
        }
        self.object.settings.pixels_per_unit = pixels_per_unit;
        self
    }

    pub fn source(mut self, canvas: &Canvas2DHandler) -> Self {
        let resolution = canvas
            .object_world()
            .borrow()
            .get::<&CanvasSettings>(canvas.get_id())
            .expect("Canvas2D handler must contain CanvasSettings.")
            .resolution;
        self.object.source = ProjectionSource(Some(canvas.get_texture()));
        let scale = self.object.settings.pixels_per_unit;
        self.object.shape.size =
            glam::vec2(resolution.0 as f32 / scale, resolution.1 as f32 / scale);
        self
    }
}

fn draw_projection(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    let shape = world.get::<&PlaneShape>(entity).unwrap();
    let size = (shape.size * 0.5).extend(1.0);
    validate_dimensions(size)?;
    let transformation = global_matrix3d(world, entity) * glam::Mat4::from_scale(size);
    validate_transformation(transformation)?;
    if transformation.determinant().abs() <= f32::EPSILON {
        return Ok(());
    }
    let source = world
        .get::<&ProjectionSource>(entity)
        .unwrap()
        .0
        .ok_or("Projection requires a source canvas.")?;
    let texture = context
        .canvas_texture(source)
        .ok_or("Projection source texture is unavailable.")?;
    let camera = context.camera;
    let mesh = context.geometry(GeometryKey::new::<PlaneShape>(0), three_d::CpuMesh::square);
    mesh.set_transformation(transformation.to_cols_array_2d().into());
    mesh.render_with_material(&ProjectionMaterial { texture }, camera, &[]);
    Ok(())
}

struct ProjectionMaterial {
    texture: glow::NativeTexture,
}

impl three_d::Material for ProjectionMaterial {
    fn id(&self) -> three_d::EffectMaterialId {
        three_d::EffectMaterialId(0)
    }

    fn fragment_shader_source(&self, _: &[&dyn three_d::Light]) -> String {
        // Skia and the output use display-encoded premultiplied RGBA. No gamma conversion here.
        // Mesh upload already flips the asset UVs into OpenGL orientation.
        "in vec2 uvs; uniform sampler2D source; out vec4 outColor;
        void main() { outColor = texture(source, uvs); }"
            .into()
    }

    fn use_uniforms(
        &self,
        program: &three_d::Program,
        _: &dyn three_d::Viewer,
        _: &[&dyn three_d::Light],
    ) {
        // Three-d 0.19 cannot borrow an externally owned Texture2D through its typed API.
        #[allow(deprecated)]
        program.use_raw_texture("source", glow::TEXTURE_2D, self.texture);
    }

    fn render_states(&self) -> three_d::RenderStates {
        render_states(true, true)
    }

    fn material_type(&self) -> three_d::MaterialType {
        three_d::MaterialType::Transparent
    }
}
