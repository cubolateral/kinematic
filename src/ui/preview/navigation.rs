use crate::ui::preview::image;
use crate::{
    editor::Editor,
    ui::inputs::{self, Action},
};

const ACTIONS: [Action; 14] = [
    Action::FocusSelection,
    Action::FrameAll,
    Action::ViewFront,
    Action::ViewBack,
    Action::ViewRight,
    Action::ViewLeft,
    Action::ViewTop,
    Action::ViewBottom,
    Action::ToggleProjection,
    Action::OrbitLeft,
    Action::OrbitRight,
    Action::OrbitUp,
    Action::OrbitDown,
    Action::ViewOpposite,
];

fn apply(editor: &mut Editor, action: Action) {
    use std::f32::consts::{FRAC_PI_2, PI};
    let step = 15.0_f32.to_radians();
    match action {
        Action::FocusSelection => editor.frame_editor_3d(true),
        Action::FrameAll => editor.frame_editor_3d(false),
        Action::ViewFront => editor.align_editor_3d(0.0, 0.0),
        Action::ViewBack => editor.align_editor_3d(PI, 0.0),
        Action::ViewRight => editor.align_editor_3d(FRAC_PI_2, 0.0),
        Action::ViewLeft => editor.align_editor_3d(-FRAC_PI_2, 0.0),
        Action::ViewTop => editor.align_editor_3d(0.0, -FRAC_PI_2),
        Action::ViewBottom => editor.align_editor_3d(0.0, FRAC_PI_2),
        Action::ToggleProjection => editor.toggle_editor_3d_projection(),
        Action::OrbitLeft => editor.orbit_editor_3d(step, 0.0),
        Action::OrbitRight => editor.orbit_editor_3d(-step, 0.0),
        Action::OrbitUp => editor.orbit_editor_3d(0.0, -step),
        Action::OrbitDown => editor.orbit_editor_3d(0.0, step),
        Action::ViewOpposite => editor.opposite_editor_3d(),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn control(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    keyboard: bool,
    active: bool,
    hovered: bool,
    min: [f32; 2],
    size: [f32; 2],
) {
    let right = dear_imgui_rs::MouseButton::Right;
    let middle = dear_imgui_rs::MouseButton::Middle;
    let looking = active && ui.is_mouse_down(right);
    let dragging = active && ui.is_mouse_down(middle);
    if looking {
        let axis = |positive, negative| {
            f32::from(inputs::active(ui, positive)) - f32::from(inputs::active(ui, negative))
        };
        let delta = if ui.is_mouse_clicked(right) {
            [0.0; 2]
        } else {
            editor.editor_mouse_delta(ui.io().mouse_delta())
        };
        if ui.io().mouse_wheel() != 0.0 {
            editor.change_editor_3d_speed(ui.io().mouse_wheel());
        }
        editor.control_editor_3d(
            delta,
            [
                axis(Action::MoveRight, Action::MoveLeft),
                axis(Action::MoveUp, Action::MoveDown),
                axis(Action::MoveForward, Action::MoveBackward),
            ],
            ui.io().delta_time(),
            inputs::active(ui, Action::CameraFast),
            inputs::active(ui, Action::CameraSlow),
        );
    } else if dragging {
        let delta = if ui.is_mouse_clicked(middle) {
            [0.0; 2]
        } else {
            editor.editor_mouse_delta(ui.io().mouse_delta())
        };
        if delta != [0.0; 2] {
            if ui.io().key_shift() {
                editor.pan_editor_3d(delta, size[1]);
            } else {
                editor.orbit_editor_3d(-delta[0] * 0.003, -delta[1] * 0.003);
            }
        }
    } else if hovered && ui.io().mouse_wheel() != 0.0 {
        editor.zoom_editor_3d(ui.io().mouse_wheel());
    }
    if looking || dragging {
        image::wrap_pointer(ui, editor, min, [min[0] + size[0], min[1] + size[1]]);
        ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeAll));
    } else if keyboard && hovered {
        for action in ACTIONS {
            if inputs::active(ui, action) {
                apply(editor, action);
            }
        }
    }
}

pub(super) fn tools(editor: &mut Editor, ui: &dear_imgui_rs::Ui, min: [f32; 2]) {
    let map = inputs::map();
    let entries = ACTIONS.map(|action| {
        let binding = map.binding(action);
        (binding.name, binding.display.as_str())
    });
    let tools = kinematic_ui::canvas::tools(
        ui,
        min,
        editor.editor_3d_camera_view(),
        editor.margin_visible(),
        &editor.editor_3d_axes(),
        super::camera_shortcuts(),
        &entries,
    );
    if tools.camera {
        editor.toggle_editor_3d_camera_view();
    }
    if tools.reset {
        editor.reset_editor_3d_camera_transform();
    }
    if tools.margin {
        editor.toggle_margin();
    }
    if let Some(axis) = tools.axis {
        editor.toggle_editor_3d_axis(axis);
    }
    if let Some(index) = tools.option {
        apply(editor, ACTIONS[index]);
    }
}

pub(super) fn gizmo(
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
    min: [f32; 2],
    size: [f32; 2],
    state: &mut kinematic_ui::canvas::orientation::State,
) {
    let inverse = editor.editor_3d_camera().camera_rotation.inverse();
    let axes =
        [glam::Vec3::X, glam::Vec3::Y, glam::Vec3::Z].map(|axis| (inverse * axis).to_array());
    let response = kinematic_ui::canvas::orientation::draw(ui, min, size, axes, state);
    if let Some(delta) = response.orbit {
        editor.orbit_editor_3d(-delta[0] * 0.01, -delta[1] * 0.01);
    }
    if let Some((axis, positive)) = response.axis {
        apply(
            editor,
            match (axis, positive) {
                (0, true) => Action::ViewRight,
                (0, false) => Action::ViewLeft,
                (1, true) => Action::ViewTop,
                (1, false) => Action::ViewBottom,
                (2, true) => Action::ViewFront,
                (2, false) => Action::ViewBack,
                _ => unreachable!(),
            },
        );
    }
}
