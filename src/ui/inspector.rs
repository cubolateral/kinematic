use crate::{
    core::{
        TrackValue,
        components::{Inspection, Name, Node},
        normalized_quaternion,
        objects::{
            CanvasSettings, ProjectionSource, SphereShape,
            appearance::{AppearanceEdit, refresh_appearance},
        },
    },
    editor::Editor,
};
use std::{ffi::CString, os::raw::c_void, ptr};

use super::widgets::{NumericValue, numeric_input_arrows, text_size};

pub(super) const WINDOW_NAME: &str = "Inspector";
const DRAG_DIRECTION_THRESHOLD: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragAxis {
    Horizontal,
    Vertical,
}

#[derive(Default)]
pub(super) struct State {
    drag: Option<(dear_imgui_rs::Id, Option<DragAxis>)>,
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    let selected = editor.get_selected_entity();
    let editing_disabled = editor.is_exporting() || editor.get_timeline().is_playing();

    ui.window(WINDOW_NAME).build(|| {
        let Some(entity) = selected else {
            ui.text_wrapped("Select an object from the Scene Tree or Timeline.");
            return;
        };

        let scene = editor.get_scene();
        let world = scene.get_world();
        let Ok(inspection) = world.get::<&Inspection>(entity) else {
            ui.text_disabled("The selected object is unavailable.");
            return;
        };
        let name = world
            .get::<&Name>(entity)
            .expect("Inspected object must contain a Name component.");
        let node = world
            .get::<&Node>(entity)
            .expect("Inspected object must contain a Node component.");

        ui.text(name.get());
        ui.same_line();
        ui.text_disabled(if node.is_activated {
            "Active."
        } else {
            "Inactive."
        });
        ui.text_disabled(format!("Entity ID: {}.", entity.to_bits()));
        ui.text_disabled(format!("Object type: {}.", inspection.object_name));
        ui.separator();
        if !node.is_activated {
            ui.text_disabled("Inactive objects cannot be edited.");
            return;
        }

        let _disabled = ui.begin_disabled_with_cond(editing_disabled);
        if let Ok(settings) = world.get::<&CanvasSettings>(entity) {
            property(
                ui,
                "Resolution",
                &format!("{} x {}.", settings.resolution.0, settings.resolution.1),
            );
        }
        if let Ok(mut sphere) = world.get::<&mut SphereShape>(entity) {
            let mut segments = sphere.segments;
            if numeric_drag(
                ui,
                state,
                "Segments",
                &mut segments,
                1.0,
                "%u",
                dear_imgui_rs::sys::ImGuiDataType_U32,
            ) {
                sphere.segments = segments.clamp(3, 256);
                scene.invalidate();
            }
        }
        if let Ok(source) = world.get::<&ProjectionSource>(entity) {
            property(
                ui,
                "Source canvas",
                &source.0.map_or_else(
                    || "Unassigned.".into(),
                    |texture| format!("{}.", texture.entity.to_bits()),
                ),
            );
        }

        let mut edits = Vec::new();
        for trackable in (inspection.get)(&world, entity) {
            ui.separator_with_text(trackable.name);

            for track in (trackable.get)() {
                let _id = ui.push_id(&format!("{}:{}", trackable.name, track.id));
                let mut value = (track.get)(&world, entity);
                let before = value.clone();
                if edit_value(ui, state, track.name, &mut value) {
                    (track.set)(&world, entity, value.clone());
                    edits.push(AppearanceEdit {
                        entity,
                        component: (trackable.type_id)(),
                        track,
                        before,
                        value,
                    });
                }
            }

            ui.spacing();
        }
        refresh_appearance(&world, &edits);
        if !edits.is_empty() {
            scene.invalidate();
        }
    });
}

fn property(ui: &dear_imgui_rs::Ui, name: &str, value: &str) {
    ui.text(name);
    ui.same_line();

    let width = text_size(ui, value)[0];
    ui.set_cursor_pos_x(ui.cursor_pos_x() + ui.content_region_avail()[0] - width);
    ui.text_disabled(value);
}

fn edit_value(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    name: &str,
    value: &mut TrackValue,
) -> bool {
    match value {
        TrackValue::Bool(v) => ui.checkbox(name, v),
        TrackValue::F32(v) => {
            let format = float_format(*v);
            numeric_drag(
                ui,
                state,
                name,
                v,
                0.01,
                &format,
                dear_imgui_rs::sys::ImGuiDataType_Float,
            )
        }
        TrackValue::Quad(v) => {
            let mut values = v.to_array();
            if edit_float_components(ui, state, name, &mut values, ["a", "b", "c", "d"]) {
                *v = values.into();
                true
            } else {
                false
            }
        }
        TrackValue::U32(v) => numeric_drag(
            ui,
            state,
            name,
            v,
            1.0,
            "%u",
            dear_imgui_rs::sys::ImGuiDataType_U32,
        ),
        TrackValue::I32(v) => numeric_drag(
            ui,
            state,
            name,
            v,
            1.0,
            "%d",
            dear_imgui_rs::sys::ImGuiDataType_S32,
        ),
        TrackValue::Vector2(v) => {
            let mut values = v.to_array();
            if edit_float_components(ui, state, name, &mut values, ["x", "y"]) {
                *v = values.into();
                true
            } else {
                false
            }
        }
        TrackValue::Vector3(v) => {
            let mut values = v.to_array();
            if edit_float_components(ui, state, name, &mut values, ["x", "y", "z"]) {
                *v = values.into();
                true
            } else {
                false
            }
        }
        TrackValue::Quaternion(v) => {
            let mut values = v.to_array();
            if edit_float_components(ui, state, name, &mut values, ["x", "y", "z", "w"]) {
                *v = normalized_quaternion(glam::Quat::from_array(values));
                true
            } else {
                false
            }
        }
        TrackValue::Color(v) => {
            let mut values = v.rgba();
            if ui.color_edit4(name, &mut values) {
                *v = crate::core::types::Color::new(values[0], values[1], values[2], values[3]);
                true
            } else {
                false
            }
        }
        TrackValue::String(v) => ui
            .input_text_multiline(name, v, [0.0, ui.frame_height() * 3.0])
            .flags(
                dear_imgui_rs::InputTextMultilineFlags::CTRL_ENTER_FOR_NEW_LINE
                    | dear_imgui_rs::InputTextMultilineFlags::WORD_WRAP,
            )
            .build(),
    }
}

fn edit_float_components<const N: usize>(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    name: &str,
    values: &mut [f32; N],
    prefixes: [&str; N],
) -> bool {
    let spacing = unsafe { ui.style().item_inner_spacing() }[0];
    let width =
        ((ui.calc_item_width() - spacing * (N.saturating_sub(1) as f32)) / N as f32).max(1.0);
    let mut changed = false;

    for (index, (value, prefix)) in values.iter_mut().zip(prefixes).enumerate() {
        if index > 0 {
            ui.same_line_with_spacing(0.0, spacing);
        }

        ui.set_next_item_width(width);
        let label = format!("##{name}:{prefix}");
        let format = format!("{prefix}: {}", float_format(*value));
        changed |= numeric_drag(
            ui,
            state,
            &label,
            value,
            0.01,
            &format,
            dear_imgui_rs::sys::ImGuiDataType_Float,
        );
    }

    ui.same_line_with_spacing(0.0, spacing);
    ui.text(name);
    changed
}

fn numeric_drag<T>(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    label: &str,
    value: &mut T,
    speed: f32,
    format: &str,
    data_type: dear_imgui_rs::sys::ImGuiDataType,
) -> bool
where
    T: NumericValue,
{
    let label = CString::new(label).expect("Inspector labels must not contain null bytes.");
    let format = CString::new(format).expect("Inspector formats must not contain null bytes.");
    let flags = dear_imgui_rs::sys::ImGuiSliderFlags_NoRoundToFormat;

    // SAFETY: The data type matches T at every call site and both strings live through the call.
    let mut changed = unsafe {
        dear_imgui_rs::sys::igDragScalar(
            label.as_ptr(),
            data_type,
            value as *mut T as *mut c_void,
            0.0,
            ptr::null(),
            ptr::null(),
            format.as_ptr(),
            flags,
        )
    };

    let text_editing = ui.is_item_active() && ui.io().want_text_input();
    if !ui.is_item_active() {
        if state.drag.is_some_and(|(id, _)| id == ui.item_id()) {
            state.drag = None;
        }
    } else if !text_editing && ui.is_mouse_down(dear_imgui_rs::MouseButton::Left) {
        let item = ui.item_id();
        if ui.is_item_activated() {
            state.drag = Some((item, None));
        }

        if let Some((_, axis)) = state.drag.as_mut().filter(|(id, _)| *id == item) {
            if axis.is_none() {
                let delta = ui.mouse_drag_delta_with_threshold(
                    dear_imgui_rs::MouseButton::Left,
                    DRAG_DIRECTION_THRESHOLD,
                );
                if delta[0].abs().max(delta[1].abs()) >= DRAG_DIRECTION_THRESHOLD {
                    *axis = Some(if delta[1].abs() > delta[0].abs() {
                        DragAxis::Vertical
                    } else {
                        DragAxis::Horizontal
                    });
                }
            }

            let amount = match *axis {
                Some(DragAxis::Vertical) => -ui.io().mouse_delta()[1] * speed,
                Some(DragAxis::Horizontal) => ui.io().mouse_delta()[0] * speed * 10.0,
                None => 0.0,
            };
            changed |= value.offset(amount);
        }
    }
    changed |= numeric_input_arrows(ui, value);

    if ui.is_item_hovered() && !text_editing {
        ui.set_mouse_cursor(Some(dear_imgui_rs::MouseCursor::ResizeAll));
    }

    changed
}

fn float_format(value: f32) -> String {
    let text = value.to_string();
    if text.contains(['e', 'E']) {
        return "%.9g".to_owned();
    }

    let decimal_places = text
        .split_once('.')
        .map_or(1, |(_, fraction)| fraction.len());
    if decimal_places > 9 {
        return "%.9g".to_owned();
    }

    format!("%.{}f", decimal_places.max(1))
}

#[cfg(test)]
mod tests {
    use super::float_format;

    #[test]
    fn float_format_keeps_only_required_decimal_places() {
        assert_eq!(float_format(1.0), "%.1f");
        assert_eq!(float_format(1.2323), "%.4f");
        assert_eq!(float_format(2.4300), "%.2f");
        assert_eq!(float_format(1e-20), "%.9g");
    }
}
