use crate::core::{components::Camera3DMode, objects::child_iter};
use crate::editor::editor::{Editor, EditorView3D};

impl EditorView3D {
    fn rotation(&self) -> glam::Quat {
        glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, self.roll)
    }

    fn pivot(&self) -> glam::Vec3 {
        self.position - self.rotation() * glam::Vec3::Z * self.orbit_distance
    }

    fn view_height(&self) -> f32 {
        match self.projection.camera_mode {
            Camera3DMode::Perspective => {
                2.0 * self.orbit_distance * (self.projection.camera_fov * 0.5).tan()
            }
            Camera3DMode::Orthogonal => self.projection.camera_fov,
        }
    }

    fn orbit(&mut self, yaw: f32, pitch: f32) {
        let pivot = self.pivot();
        self.yaw += yaw;
        self.pitch =
            (self.pitch + pitch).clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
        self.position = pivot + self.rotation() * glam::Vec3::Z * self.orbit_distance;
    }

    fn toggle_projection(&mut self) {
        let height = self.view_height();
        match self.projection.camera_mode {
            Camera3DMode::Perspective => {
                self.perspective_fov = self.projection.camera_fov;
                self.projection.camera_mode = Camera3DMode::Orthogonal;
                self.projection.camera_fov = height;
            }
            Camera3DMode::Orthogonal => {
                let pivot = self.pivot();
                self.orbit_distance = height / (2.0 * (self.perspective_fov * 0.5).tan());
                self.position = pivot + self.rotation() * glam::Vec3::Z * self.orbit_distance;
                self.projection.camera_mode = Camera3DMode::Perspective;
                self.projection.camera_fov = self.perspective_fov;
                self.projection.camera_far = self
                    .projection
                    .camera_far
                    .max(self.orbit_distance * 2.0 + self.projection.camera_near);
            }
        }
    }

    fn frame(&mut self, min: glam::Vec3, max: glam::Vec3, aspect: f32) {
        let radius = ((max - min).length() * 0.5).max(0.01) * 1.1;
        if self.projection.camera_mode == Camera3DMode::Perspective {
            self.perspective_fov = self.projection.camera_fov;
        }
        let half_fov = self.perspective_fov * 0.5;
        let limiting_angle = half_fov.min((half_fov.tan() * aspect).atan());
        self.orbit_distance =
            (radius / limiting_angle.sin()).max(radius + self.projection.camera_near * 2.0);
        self.position = (min + max) * 0.5 + self.rotation() * glam::Vec3::Z * self.orbit_distance;
        if self.projection.camera_mode == Camera3DMode::Orthogonal {
            self.projection.camera_fov = 2.0 * radius / aspect.min(1.0);
        }
        self.projection.camera_far = self
            .projection
            .camera_far
            .max(self.orbit_distance + radius * 2.0);
    }
}

impl Editor {
    fn release_editor_3d_camera(&mut self) {
        if self.editor_view_3d.canvas_view {
            let camera = self.editor_3d_camera();
            let (yaw, pitch, roll) = camera.camera_rotation.to_euler(glam::EulerRot::YXZ);
            let view = &mut self.editor_view_3d;
            view.position = camera.camera_position;
            view.yaw = yaw;
            view.pitch = pitch;
            view.roll = roll;
            if camera.camera_mode == Camera3DMode::Perspective {
                view.perspective_fov = camera.camera_fov;
            }
            view.projection = camera;
            view.canvas_view = false;
        }
    }

    pub(crate) fn control_editor_3d(
        &mut self,
        look: [f32; 2],
        movement: [f32; 3],
        delta: f32,
        fast: bool,
        slow: bool,
    ) {
        if look == [0.0; 2] && (movement == [0.0; 3] || delta <= 0.0) {
            return;
        }
        self.release_editor_3d_camera();
        let view = &mut self.editor_view_3d;
        view.yaw -= look[0] * 0.003;
        view.pitch = (view.pitch - look[1] * 0.003).clamp(-1.55, 1.55);
        let speed = view.speed
            * if slow {
                0.2
            } else if fast {
                4.0
            } else {
                1.0
            };
        view.position += view.rotation()
            * glam::vec3(movement[0], movement[1], -movement[2]).normalize_or_zero()
            * speed
            * delta.clamp(0.0, 0.1);
        self.editor_3d_rendered = None;
    }

    pub(crate) fn orbit_editor_3d(&mut self, yaw: f32, pitch: f32) {
        self.release_editor_3d_camera();
        self.editor_view_3d.orbit(yaw, pitch);
        self.editor_3d_rendered = None;
    }

    pub(crate) fn pan_editor_3d(&mut self, delta: [f32; 2], height: f32) {
        self.release_editor_3d_camera();
        let view = &mut self.editor_view_3d;
        view.position +=
            view.rotation() * glam::vec3(-delta[0], delta[1], 0.0) * view.view_height()
                / height.max(1.0);
        self.editor_3d_rendered = None;
    }

    pub(crate) fn zoom_editor_3d(&mut self, wheel: f32) {
        self.release_editor_3d_camera();
        let view = &mut self.editor_view_3d;
        let factor = (-wheel * 0.15).exp();
        if view.projection.camera_mode == Camera3DMode::Orthogonal {
            view.projection.camera_fov = (view.projection.camera_fov * factor).clamp(0.001, 1.0e6);
        } else {
            let pivot = view.pivot();
            let minimum = view.projection.camera_near * 2.0;
            view.orbit_distance = (view.orbit_distance * factor).clamp(minimum, minimum.max(1.0e6));
            view.projection.camera_far = view
                .projection
                .camera_far
                .max(view.orbit_distance * 2.0 + view.projection.camera_near);
            view.position = pivot + view.rotation() * glam::Vec3::Z * view.orbit_distance;
        }
        self.editor_3d_rendered = None;
    }

    pub(crate) fn change_editor_3d_speed(&mut self, wheel: f32) {
        self.editor_view_3d.speed =
            (self.editor_view_3d.speed * (wheel * 0.2).exp()).clamp(0.01, 10000.0);
    }

    pub(crate) fn toggle_editor_3d_projection(&mut self) {
        self.release_editor_3d_camera();
        self.editor_view_3d.toggle_projection();
        self.editor_3d_rendered = None;
    }

    pub(crate) fn opposite_editor_3d(&mut self) {
        self.release_editor_3d_camera();
        let pitch = self.editor_view_3d.pitch;
        self.orbit_editor_3d(std::f32::consts::PI, -2.0 * pitch);
    }

    pub(crate) fn align_editor_3d(&mut self, yaw: f32, pitch: f32) {
        self.release_editor_3d_camera();
        let view = &mut self.editor_view_3d;
        let pivot = view.pivot();
        view.yaw = yaw;
        view.pitch = pitch;
        view.roll = 0.0;
        view.position = pivot + view.rotation() * glam::Vec3::Z * view.orbit_distance;
        if view.projection.camera_mode == Camera3DMode::Perspective {
            view.toggle_projection();
        }
        self.editor_3d_rendered = None;
    }

    pub(crate) fn frame_editor_3d(&mut self, selection: bool) {
        let canvas = self.editor_3d_canvas();
        let scene = &self.scenes[self.active_scene].scene;
        let targets = if selection {
            let Some(entity) = self.selected_entity() else {
                return;
            };
            if entity == canvas {
                child_iter(&scene.world(), canvas).collect::<Vec<_>>()
            } else {
                vec![entity]
            }
        } else {
            child_iter(&scene.world(), canvas).collect::<Vec<_>>()
        };
        let mut min = glam::Vec3::splat(f32::INFINITY);
        let mut max = glam::Vec3::splat(f32::NEG_INFINITY);
        for target in targets {
            if let Some(segments) = scene.editor_3d_selection_segments(canvas, target) {
                for point in segments.into_iter().flatten() {
                    min = min.min(point);
                    max = max.max(point);
                }
            }
        }
        if !min.is_finite() || !max.is_finite() {
            return;
        }
        self.release_editor_3d_camera();
        let size = self.editor_3d.size();
        self.editor_view_3d
            .frame(min, max, size.0.max(1) as f32 / size.1.max(1) as f32);
        self.editor_3d_rendered = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_and_projection_changes_preserve_the_pivot_and_framing() {
        let mut view = EditorView3D::default();
        let pivot = view.pivot();
        let height = view.view_height();
        view.orbit(0.4, 0.2);
        assert!(view.pivot().abs_diff_eq(pivot, 1e-5));
        view.toggle_projection();
        assert!((view.view_height() - height).abs() < 1e-5);
        view.projection.camera_fov *= 0.5;
        let height = view.view_height();
        view.toggle_projection();
        assert!(view.pivot().abs_diff_eq(pivot, 1e-5));
        assert!((view.view_height() - height).abs() < 1e-5);
    }

    #[test]
    fn framing_fits_transformed_bounds_in_portrait_and_orthographic_views() {
        let min = glam::vec3(-8.0, 2.0, -3.0);
        let max = glam::vec3(2.0, 9.0, 5.0);
        for aspect in [0.5, 2.0] {
            let mut view = EditorView3D::default();
            for orthographic in [false, true] {
                if orthographic {
                    view.toggle_projection();
                }
                view.frame(min, max, aspect);
                assert!(view.pivot().abs_diff_eq((min + max) * 0.5, 1e-5));
                let inverse = view.rotation().inverse();
                for x in [min.x, max.x] {
                    for y in [min.y, max.y] {
                        for z in [min.z, max.z] {
                            let point = inverse * (glam::vec3(x, y, z) - view.position);
                            let height = if orthographic {
                                view.view_height()
                            } else {
                                -point.z * 2.0 * (view.projection.camera_fov * 0.5).tan()
                            };
                            assert!(point.x.abs() <= height * aspect * 0.5);
                            assert!(point.y.abs() <= height * 0.5);
                            assert!(-point.z > view.projection.camera_near);
                        }
                    }
                }
            }
        }
    }
}
