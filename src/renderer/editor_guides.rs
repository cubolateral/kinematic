use crate::core::components::Camera3D;
use three_d::{Geometry, Viewer};

pub(crate) struct EditorGuides3D {
    pub axes: [bool; 3],
    pub canvas_camera: Option<(Camera3D, (u32, u32))>,
    pub selection: Option<Vec<[glam::Vec3; 2]>>,
}

pub(in crate::renderer) struct EditorGuideRenderer {
    mesh: Option<three_d::Mesh>,
    visible: three_d::ColorMaterial,
    occluded: three_d::ColorMaterial,
}

impl EditorGuideRenderer {
    pub(in crate::renderer) fn new() -> Self {
        Self {
            mesh: None,
            visible: guide_material(three_d::DepthTest::LessOrEqual, three_d::Srgba::WHITE),
            occluded: guide_material(
                three_d::DepthTest::Always,
                three_d::Srgba::new(175, 190, 220, 42),
            ),
        }
    }

    pub(in crate::renderer) fn render(
        &mut self,
        context: &three_d::Context,
        camera: &three_d::Camera,
        view_camera: &Camera3D,
        guides: &EditorGuides3D,
    ) -> Result<(), String> {
        let (positions, colors) = guide_mesh(view_camera, camera.viewport().height, guides);
        if positions.is_empty() {
            self.mesh = None;
            return Ok(());
        }

        let positions = positions
            .into_iter()
            .map(|point| three_d::vec3(point.x, point.y, point.z))
            .collect::<Vec<_>>();
        let colors = colors
            .into_iter()
            .map(|color| three_d::vec4(color[0], color[1], color[2], color[3]))
            .collect::<Vec<_>>();
        if self
            .mesh
            .as_ref()
            .is_none_or(|mesh| mesh.vertex_count() != positions.len() as u32)
        {
            let cpu = three_d::CpuMesh {
                positions: three_d::Positions::F32(positions),
                ..Default::default()
            };
            self.mesh = Some(three_d::Mesh::new(context, &cpu));
        } else {
            self.mesh
                .as_mut()
                .unwrap()
                .set_positions(&positions)
                .map_err(|error| error.to_string())?;
        }
        let mesh = self.mesh.as_mut().unwrap();
        mesh.set_colors(&colors)
            .map_err(|error| error.to_string())?;

        mesh.render_with_material(&self.occluded, camera, &[]);
        mesh.render_with_material(&self.visible, camera, &[]);
        Ok(())
    }
}

fn guide_material(depth_test: three_d::DepthTest, color: three_d::Srgba) -> three_d::ColorMaterial {
    use three_d::{BlendEquationType as E, BlendMultiplierType as M};

    three_d::ColorMaterial {
        color,
        render_states: three_d::RenderStates {
            write_mask: three_d::WriteMask::COLOR,
            depth_test,
            blend: three_d::Blend::Enabled {
                source_rgb_multiplier: M::SrcAlpha,
                source_alpha_multiplier: M::One,
                destination_rgb_multiplier: M::OneMinusSrcAlpha,
                destination_alpha_multiplier: M::OneMinusSrcAlpha,
                rgb_equation: E::Add,
                alpha_equation: E::Add,
            },
            ..Default::default()
        },
        is_transparent: true,
        ..Default::default()
    }
}

fn guide_mesh(
    view_camera: &Camera3D,
    viewport_height: u32,
    guides: &EditorGuides3D,
) -> (Vec<glam::Vec3>, Vec<[f32; 4]>) {
    let camera_scale = view_camera.camera_position.length().max(1.0);
    let guide_scale = 10.0_f32.powf(camera_scale.log10().floor());
    let guide_extent = guide_scale * 100.0;
    let mut positions = Vec::new();
    let mut colors = Vec::new();

    let axis_extent = guide_extent.max(100.0);
    for (enabled, from, to, color) in [
        (
            guides.axes[0],
            glam::vec3(-axis_extent, 0.0, 0.0),
            glam::vec3(axis_extent, 0.0, 0.0),
            [1.0, 0.25, 0.25, 1.0],
        ),
        (
            guides.axes[1],
            glam::vec3(0.0, -axis_extent, 0.0),
            glam::vec3(0.0, axis_extent, 0.0),
            [0.25, 1.0, 0.25, 1.0],
        ),
        (
            guides.axes[2],
            glam::vec3(0.0, 0.0, -axis_extent),
            glam::vec3(0.0, 0.0, axis_extent),
            [0.3, 0.5, 1.0, 1.0],
        ),
    ] {
        if enabled {
            add_segment(
                &mut positions,
                &mut colors,
                view_camera,
                viewport_height,
                from,
                to,
                color,
            );
        }
    }

    if let Some((camera, resolution)) = &guides.canvas_camera {
        let distance = 1.0;
        let half_height = (camera.camera_fov * 0.5).tan() * distance;
        let half_width = half_height * resolution.0.max(1) as f32 / resolution.1.max(1) as f32;
        let transform = camera.matrix();
        let origin = camera.camera_position;
        let corners = [
            glam::vec3(-half_width, half_height, -distance),
            glam::vec3(half_width, half_height, -distance),
            glam::vec3(half_width, -half_height, -distance),
            glam::vec3(-half_width, -half_height, -distance),
        ]
        .map(|point| transform.transform_point3(point));
        for index in 0..4 {
            for [from, to] in [
                [origin, corners[index]],
                [corners[index], corners[(index + 1) % 4]],
            ] {
                add_segment(
                    &mut positions,
                    &mut colors,
                    view_camera,
                    viewport_height,
                    from,
                    to,
                    [1.0, 1.0, 1.0, 1.0],
                );
            }
        }
    }

    if let Some(segments) = &guides.selection {
        for [from, to] in segments {
            add_segment(
                &mut positions,
                &mut colors,
                view_camera,
                viewport_height,
                *from,
                *to,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    }

    (positions, colors)
}

fn add_segment(
    positions: &mut Vec<glam::Vec3>,
    colors: &mut Vec<[f32; 4]>,
    camera: &Camera3D,
    viewport_height: u32,
    from: glam::Vec3,
    to: glam::Vec3,
    color: [f32; 4],
) {
    let rotation = crate::core::normalized_quaternion(camera.camera_rotation);
    let right = rotation * glam::Vec3::X;
    let up = rotation * glam::Vec3::Y;
    let forward = rotation * -glam::Vec3::Z;
    let Some((from, to)) = clip_segment_to_camera(camera, forward, from, to) else {
        return;
    };
    let direction = to - from;
    if !direction.is_finite() || direction.length_squared() <= f32::EPSILON {
        return;
    }
    let relative = |point: glam::Vec3| point - camera.camera_position;
    let screen = |point: glam::Vec3| {
        let point = relative(point);
        let depth = point.dot(forward);
        glam::vec2(point.dot(right), point.dot(up)) / depth
    };
    let screen_direction = screen(to) - screen(from);
    if !screen_direction.is_finite() || screen_direction.length_squared() <= f32::EPSILON {
        return;
    }
    let screen_side = glam::vec2(-screen_direction.y, screen_direction.x).normalize();
    let side = right * screen_side.x + up * screen_side.y;
    let world_per_pixel = |point: glam::Vec3| {
        let depth = relative(point).dot(forward);
        2.0 * depth * (camera.camera_fov * 0.5).tan() / viewport_height.max(1) as f32
    };
    let from_side = side * world_per_pixel(from) * 0.5;
    let to_side = side * world_per_pixel(to) * 0.5;
    let vertices = [
        from - from_side,
        from + from_side,
        to + to_side,
        from - from_side,
        to + to_side,
        to - to_side,
    ];
    positions.extend(vertices);
    colors.extend([color; 6]);
}

fn clip_segment_to_camera(
    camera: &Camera3D,
    forward: glam::Vec3,
    from: glam::Vec3,
    to: glam::Vec3,
) -> Option<(glam::Vec3, glam::Vec3)> {
    let from_depth = (from - camera.camera_position).dot(forward);
    let to_depth = (to - camera.camera_position).dot(forward);
    let depth_delta = to_depth - from_depth;
    if depth_delta.abs() <= f32::EPSILON {
        return (camera.camera_near..=camera.camera_far)
            .contains(&from_depth)
            .then_some((from, to));
    }

    let near = (camera.camera_near - from_depth) / depth_delta;
    let far = (camera.camera_far - from_depth) / depth_delta;
    let start = near.min(far).clamp(0.0, 1.0);
    let end = near.max(far).clamp(0.0, 1.0);
    (start < end).then(|| {
        let direction = to - from;
        (from + direction * start, from + direction * end)
    })
}

#[cfg(test)]
mod tests {
    use super::{EditorGuideRenderer, EditorGuides3D, add_segment, guide_mesh};
    use crate::core::components::Camera3D;
    use three_d::{Blend, BlendMultiplierType as M};

    #[test]
    fn guide_mesh_contains_two_triangles_per_enabled_segment() {
        let guides = EditorGuides3D {
            axes: [true, false, false],
            canvas_camera: None,
            selection: None,
        };
        let (positions, colors) = guide_mesh(&Camera3D::default(), 720, &guides);
        assert_eq!(positions.len(), 6);
        assert_eq!(colors.len(), positions.len());
    }

    #[test]
    fn occluded_guides_use_xray_depth_without_writing_depth() {
        let renderer = EditorGuideRenderer::new();
        assert_eq!(
            renderer.occluded.render_states.depth_test,
            three_d::DepthTest::Always
        );
        assert!(!renderer.occluded.render_states.write_mask.depth);
        assert_eq!(
            renderer.visible.render_states.depth_test,
            three_d::DepthTest::LessOrEqual
        );
        let Blend::Enabled {
            source_alpha_multiplier,
            destination_alpha_multiplier,
            ..
        } = renderer.visible.render_states.blend
        else {
            panic!("Editor guides must use alpha blending.");
        };
        assert_eq!(source_alpha_multiplier, M::One);
        assert_eq!(destination_alpha_multiplier, M::OneMinusSrcAlpha);
    }

    #[test]
    fn guide_mesh_is_clipped_in_front_of_the_camera() {
        let camera = Camera3D::default();
        let mut positions = Vec::new();
        let mut colors = Vec::new();
        add_segment(
            &mut positions,
            &mut colors,
            &camera,
            720,
            glam::vec3(1.0, 0.0, -100.0),
            glam::vec3(1.0, 0.0, 100.0),
            [1.0; 4],
        );
        let forward = -glam::Vec3::Z;
        assert!(!positions.is_empty());
        assert!(positions.iter().all(|point| {
            (*point - camera.camera_position).dot(forward) >= camera.camera_near - 0.0001
        }));
    }

    #[test]
    fn guide_mesh_stays_one_pixel_wide_at_different_depths() {
        let mut camera = Camera3D::default();
        camera.camera_position = glam::Vec3::ZERO;
        let viewport_height = 720;
        let mut positions = Vec::new();
        let mut colors = Vec::new();
        add_segment(
            &mut positions,
            &mut colors,
            &camera,
            viewport_height,
            glam::vec3(-1.0, 0.0, -1.0),
            glam::vec3(1.0, 0.0, -10.0),
            [1.0; 4],
        );
        let projected_y = |point: glam::Vec3| {
            point.y / -point.z * viewport_height as f32 / (2.0 * (camera.camera_fov * 0.5).tan())
        };
        assert!((projected_y(positions[1]) - projected_y(positions[0]) - 1.0).abs() < 0.001);
        assert!((projected_y(positions[2]) - projected_y(positions[5]) - 1.0).abs() < 0.001);
    }
}
