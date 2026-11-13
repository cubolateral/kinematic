use crate::core::components::{Camera2D, Camera3D, Camera3DMode};
use crate::editor::editor::{Editor, EditorView2D};
use kinematic_ui::canvas::{navigation_lerp, navigation_lerp_factor};

fn lerp_angle(current: f32, target: f32, weight: f32) -> f32 {
    let difference = (target - current + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;

    navigation_lerp(current, current + difference, weight)
}

fn smooth_camera(current: &mut Camera3D, target: &Camera3D, distance: f32, weight: f32) {
    if current.camera_mode != target.camera_mode {
        let height = match current.camera_mode {
            Camera3DMode::Perspective => 2.0 * distance * (current.camera_fov * 0.5).tan(),
            Camera3DMode::Orthogonal => current.camera_fov,
        };

        current.camera_fov = match target.camera_mode {
            Camera3DMode::Perspective => 2.0 * (height / (2.0 * distance)).atan(),
            Camera3DMode::Orthogonal => height,
        };
        current.camera_mode = target.camera_mode;
    }

    for axis in 0..3 {
        current.camera_position[axis] = navigation_lerp(
            current.camera_position[axis],
            target.camera_position[axis],
            weight,
        );
    }

    current.camera_rotation = current
        .camera_rotation
        .slerp(target.camera_rotation, weight);

    if current
        .camera_rotation
        .abs_diff_eq(target.camera_rotation, 1e-5)
        || current
            .camera_rotation
            .abs_diff_eq(-target.camera_rotation, 1e-5)
    {
        current.camera_rotation = target.camera_rotation;
    }

    current.camera_fov = navigation_lerp(current.camera_fov, target.camera_fov, weight);
    current.camera_near = target.camera_near;
    current.camera_far = target.camera_far;
}

impl Editor {
    fn canvas_camera_2d(&self) -> Camera2D {
        self.scenes[self.active_scene]
            .scene
            .world()
            .get::<&Camera2D>(self.editor_2d_canvas())
            .map(|camera| (*camera).clone())
            .unwrap_or_default()
    }

    fn target_editor_2d_view(&self) -> EditorView2D {
        let mut view = self.editor_view_2d;

        if view.canvas_view {
            let camera = self.canvas_camera_2d();

            view.rotation = -camera.camera_rotation;
            view.zoom = camera.camera_zoom * self.camera_2d_scale;
            view.pan = (-(glam::Mat2::from_angle(view.rotation) * camera.camera_position)
                * view.zoom)
                .to_array();
        }

        view
    }

    pub(super) fn displayed_editor_2d_view(&self) -> EditorView2D {
        let mut view = self
            .displayed_2d
            .unwrap_or_else(|| self.target_editor_2d_view());

        view.correction = self.editor_view_2d.correction;
        view.canvas_view = self.editor_view_2d.canvas_view;

        if view.canvas_view {
            let camera = self.canvas_camera_2d();
            let position = glam::Mat2::from_angle(view.rotation) * camera.camera_position;

            view.pan = (glam::Vec2::from_array(view.pan) + position * view.zoom).to_array();
            view.zoom /= camera.camera_zoom;
            view.rotation += camera.camera_rotation;
        }

        view
    }

    pub(super) fn smooth_editor_navigation(&mut self, delta: f32) {
        let weight = navigation_lerp_factor(delta);
        let target = self.target_editor_2d_view();
        let view = self.displayed_2d.get_or_insert(target);

        for axis in 0..2 {
            view.pan[axis] = navigation_lerp(view.pan[axis], target.pan[axis], weight);
        }

        view.zoom = navigation_lerp(view.zoom, target.zoom, weight);
        view.rotation = lerp_angle(view.rotation, target.rotation, weight);

        let target = self.target_editor_3d_camera();
        let view = self.displayed_3d.get_or_insert_with(|| target.clone());
        let previous = view.clone();

        smooth_camera(view, &target, self.editor_view_3d.orbit_distance, weight);

        let mut target_projection =
            crate::core::objects::camera_projection(&target, self.editor_3d.size())
                .expect("Editor camera projection must be valid.");

        // Normalize homogeneous scale so projection changes preserve framing at the orbit pivot.
        if target.camera_mode == Camera3DMode::Perspective {
            target_projection *= self.editor_view_3d.orbit_distance.recip();
        }

        let projection = self
            .displayed_projection_3d
            .get_or_insert(target_projection);
        let previous_projection = *projection;
        let values = projection.to_cols_array();
        let target_values = target_projection.to_cols_array();

        *projection = glam::Mat4::from_cols_array(&std::array::from_fn(|axis| {
            navigation_lerp(values[axis], target_values[axis], weight)
        }));

        if *projection != previous_projection
            || view.camera_position != previous.camera_position
            || view.camera_rotation != previous.camera_rotation
            || view.camera_fov != previous.camera_fov
            || view.camera_mode != previous.camera_mode
        {
            self.editor_3d_rendered = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothing_is_independent_of_frame_rate_and_uses_the_short_rotation() {
        let target = Camera3D {
            camera_position: glam::vec3(8.0, -2.0, 5.0),
            camera_rotation: glam::Quat::from_rotation_y(-179.0_f32.to_radians()),
            camera_fov: 1.2,
            ..Default::default()
        };
        let start = Camera3D {
            camera_rotation: glam::Quat::from_rotation_y(179.0_f32.to_radians()),
            ..Default::default()
        };
        let mut at_30 = start.clone();
        let mut at_120 = start;

        smooth_camera(&mut at_30, &target, 5.0, navigation_lerp_factor(1.0 / 30.0));

        for _ in 0..4 {
            smooth_camera(
                &mut at_120,
                &target,
                5.0,
                navigation_lerp_factor(1.0 / 120.0),
            );
        }

        assert!(
            at_30
                .camera_position
                .abs_diff_eq(at_120.camera_position, 1e-5)
        );
        assert!(
            at_30
                .camera_rotation
                .abs_diff_eq(at_120.camera_rotation, 1e-5)
        );
        assert!((at_30.camera_fov - at_120.camera_fov).abs() < 1e-5);
        assert!(at_30.camera_rotation.angle_between(target.camera_rotation) < 0.01);
        assert_eq!(navigation_lerp_factor(0.0), 0.0);
        assert_eq!(
            navigation_lerp_factor(1.0 / 60.0),
            kinematic_ui::canvas::NAVIGATION_LERP_WEIGHT
        );
    }
}
