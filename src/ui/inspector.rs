use crate::{
    core::{
        TrackChoices, TrackInfo, TrackLimits, TrackValue,
        components::{Animation, Inspection, Name, TreeNode},
        normalized_quaternion,
        objects::{CanvasSettings, ProjectionSource, SphereShape},
    },
    editor::{Editor, InspectorTrack},
};
use std::{collections::HashMap, ffi::CString, os::raw::c_void, ptr};

use super::{
    icons, properties,
    widgets::{NumericValue, numeric_input_arrows},
};

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
    rotations: HashMap<dear_imgui_rs::Id, RotationEdit>,
}

struct RotationEdit {
    quaternion: glam::Quat,
    euler: [f32; 3],
    mode: usize,
}

impl RotationEdit {
    fn new(quaternion: glam::Quat) -> Self {
        Self {
            quaternion,
            euler: euler_degrees(quaternion),
            mode: 0,
        }
    }

    fn sync(&mut self, quaternion: glam::Quat) {
        if self.quaternion != quaternion {
            self.quaternion = quaternion;
            self.euler = euler_degrees(quaternion);
        }
    }
}

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui, state: &mut State) {
    let selected = editor.selected_object();
    let is_exporting = editor.is_exporting();
    let is_playing = editor.timeline_mut().is_playing();

    ui.window(WINDOW_NAME).build(|| {
        let Some((scene_index, entity)) = selected else {
            ui.text_wrapped("Select an object from the Scene Tree or Timeline.");

            return;
        };

        let _object_id = ui.push_id(&format!("{scene_index}:{}", entity.to_bits()));

        if let Some((persistent, fields)) = editor.lua_inspector_values(scene_index, entity) {
            ui.text_disabled(if persistent {
                "Base values apply before animation."
            } else {
                "Temporary edits. Add an explicit ID to persist."
            });

            let _disabled = ui.begin_disabled_with_cond(is_exporting);

            for (target, script, override_value, evaluated) in fields {
                let _id = ui.push_id(target.name());
                let mut value = override_value.clone().unwrap_or_else(|| script.clone());
                let track = match &target {
                    crate::core::TrackTarget::Property { info, .. } => Some(*info),
                    _ => None,
                };
                let edit = edit_value(
                    ui,
                    state,
                    target.name(),
                    &mut value,
                    track,
                    override_value.is_some(),
                );

                if edit.changed {
                    value = target.clamp(value);

                    let valid = match &target {
                        crate::core::TrackTarget::Dynamic(schema) => {
                            schema.validate(&value).is_ok()
                        }
                        _ => value.validate().is_ok(),
                    };

                    if valid {
                        editor.set_lua_override(scene_index, entity, &target, Some(value));
                    }
                }

                if edit.reset {
                    editor.set_lua_override(scene_index, entity, &target, None);
                }

                if override_value.is_some() {
                    ui.text_disabled(format!("Script: {script}. Override saved."));
                }

                if evaluated != override_value.unwrap_or(script) {
                    ui.text_disabled(format!("Evaluated: {evaluated}."));
                }
            }

            return;
        }

        let (scene, inspector_edits) = editor.inspector_scene_mut(scene_index);
        let world = scene.world();

        let Ok(inspection) = world.get::<&Inspection>(entity) else {
            ui.text_disabled("The selected object is unavailable.");

            return;
        };

        let name = world
            .get::<&Name>(entity)
            .expect("Inspected object must contain a Name component.");
        let node = world
            .get::<&TreeNode>(entity)
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

        {
            let _disabled = ui.begin_disabled_with_cond(is_exporting || is_playing);

            if let Ok(settings) = world.get::<&CanvasSettings>(entity) {
                property(
                    ui,
                    "Resolution",
                    &format!("{} x {}.", settings.resolution.0, settings.resolution.1),
                );
            }

            if let Ok(mut sphere) = world.get::<&mut SphereShape>(entity) {
                let mut segments = sphere.segments;

                if properties::row(ui, "Segments", false, || {
                    numeric_drag(
                        ui,
                        state,
                        "##value",
                        &mut segments,
                        1.0,
                        "%u",
                        dear_imgui_rs::sys::ImGuiDataType_U32,
                        Some(&3),
                        Some(&256),
                    )
                })
                .changed
                {
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
        }

        let mut changed = false;

        for trackable in inspection.trackables(&world, entity) {
            ui.separator_with_text(trackable.name);

            for track in (trackable.get)() {
                let _id = ui.push_id(&format!("{}:{}", trackable.name, track.id));
                let component = (trackable.type_id)();
                let animated = world
                    .get::<&Animation>(entity)
                    .is_ok_and(|animation| animation.animates(component, track));
                let _disabled = ui.begin_disabled_with_cond(is_exporting || is_playing && animated);
                let key = InspectorTrack::new(entity, component, track);
                let show_reset = inspector_edits.contains(&key);
                let mut value = (track.get)(&world, entity);
                let before = value.clone();
                let label = if animated {
                    format!("{} {}", icons::DIAMOND, track.name)
                } else {
                    track.name.to_owned()
                };
                let edit = edit_value(ui, state, &label, &mut value, Some(track), show_reset);

                if edit.changed {
                    value = track.clamp(value);

                    (track.set)(&world, entity, value.clone());

                    inspector_edits.record(
                        key.clone(),
                        track,
                        before.clone(),
                        value.clone(),
                        animated,
                    );

                    changed = true;
                }

                if edit.reset {
                    inspector_edits.reset(scene, key.clone());
                }

                if animated && edit.hovered {
                    ui.tooltip_text(if is_playing {
                        "Animated track. Pause playback to edit it."
                    } else {
                        "Animated track."
                    });
                }
            }

            ui.spacing();
        }

        let mut uniform_changed = false;
        let uniforms = crate::core::objects::shader_uniforms(&world, entity);

        if !uniforms.is_empty() {
            ui.separator_with_text("Shader uniforms");
        }

        for (name, mut value) in uniforms {
            let _id = ui.push_id(&format!("uniform:{name}"));
            let animated = world
                .get::<&Animation>(entity)
                .is_ok_and(|animation| animation.animates_uniform(&name));
            let _disabled = ui.begin_disabled_with_cond(is_exporting || is_playing && animated);
            let key = InspectorTrack::uniform(entity, &name);
            let show_reset = inspector_edits.contains(&key);
            let before = value.clone();
            let label = if animated {
                format!("{} {name}", icons::DIAMOND)
            } else {
                name.clone()
            };
            let edit = edit_value(ui, state, &label, &mut value, None, show_reset);

            if edit.changed {
                crate::core::objects::set_shader_uniform(&world, entity, &name, value.clone())
                    .unwrap_or_else(|error| panic!("{error}"));

                inspector_edits.record_uniform(key.clone(), before, value, animated);

                uniform_changed = true;
            }

            if edit.reset {
                inspector_edits.reset(scene, key.clone());
            }

            if animated && edit.hovered {
                ui.tooltip_text(if is_playing {
                    "Animated uniform. Pause playback to edit it."
                } else {
                    "Animated uniform."
                });
            }
        }

        if changed || uniform_changed {
            scene.invalidate();
        }
    });
}

fn property(ui: &dear_imgui_rs::Ui, name: &str, value: &str) {
    properties::row(ui, name, false, || {
        ui.text_disabled(value);

        false
    });
}

fn edit_value(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    name: &str,
    value: &mut TrackValue,
    track: Option<&TrackInfo>,
    show_reset: bool,
) -> properties::Edit {
    properties::row(ui, name, show_reset, || {
        edit_value_input(ui, state, "##value", value, track)
    })
}

fn edit_value_input(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    name: &str,
    value: &mut TrackValue,
    track: Option<&TrackInfo>,
) -> bool {
    match value {
        TrackValue::Bool(v) => ui.checkbox(if *v { "On###value" } else { "Off###value" }, v),
        TrackValue::Enum(v) => {
            let Some(track) = track else {
                return false;
            };

            let TrackChoices::Enum(variants) = track.choices else {
                return false;
            };

            let Some(mut current) = variants.iter().position(|variant| variant == v) else {
                return false;
            };

            if ui.combo_simple_string(name, &mut current, variants) {
                *v = variants[current];

                true
            } else {
                false
            }
        }
        TrackValue::F32(v) => {
            let format = float_format(*v);
            let (min, max) = match track.map(|track| track.limits) {
                Some(TrackLimits::F32 { min, max }) => (min, max),
                _ => (None, None),
            };

            numeric_drag(
                ui,
                state,
                name,
                v,
                0.01,
                &format,
                dear_imgui_rs::sys::ImGuiDataType_Float,
                min.as_ref(),
                max.as_ref(),
            )
        }
        TrackValue::Quad(v) => {
            let mut values = v.to_array();

            if edit_float_components(ui, state, name, &mut values, ["a", "b", "c", "d"], "") {
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
            match track.map(|track| &track.limits) {
                Some(TrackLimits::U32 { min, .. }) => min.as_ref(),
                _ => None,
            },
            match track.map(|track| &track.limits) {
                Some(TrackLimits::U32 { max, .. }) => max.as_ref(),
                _ => None,
            },
        ),
        TrackValue::I32(v) => numeric_drag(
            ui,
            state,
            name,
            v,
            1.0,
            "%d",
            dear_imgui_rs::sys::ImGuiDataType_S32,
            match track.map(|track| &track.limits) {
                Some(TrackLimits::I32 { min, .. }) => min.as_ref(),
                _ => None,
            },
            match track.map(|track| &track.limits) {
                Some(TrackLimits::I32 { max, .. }) => max.as_ref(),
                _ => None,
            },
        ),
        TrackValue::Vector2(v) => {
            let mut values = v.to_array();

            if edit_float_components(ui, state, name, &mut values, ["x", "y"], "") {
                *v = values.into();

                true
            } else {
                false
            }
        }
        TrackValue::Vector3(v) => {
            let mut values = v.to_array();

            if edit_float_components(ui, state, name, &mut values, ["x", "y", "z"], "") {
                *v = values.into();

                true
            } else {
                false
            }
        }
        TrackValue::Quaternion(v) => {
            let id = ui.get_id("rotation_mode");
            let rotation = state
                .rotations
                .entry(id)
                .or_insert_with(|| RotationEdit::new(*v));

            rotation.sync(*v);

            let mut mode = rotation.mode;
            let changed = if mode == 0 {
                let mut values = rotation.euler;

                if edit_float_components(ui, state, name, &mut values, ["x", "y", "z"], "°") {
                    *v = quaternion_from_degrees(values);

                    let rotation = state.rotations.get_mut(&id).unwrap();

                    rotation.quaternion = *v;
                    rotation.euler = values;

                    true
                } else {
                    false
                }
            } else {
                let mut values = v.to_array();

                if edit_float_components(ui, state, name, &mut values, ["x", "y", "z", "w"], "") {
                    *v = normalized_quaternion(glam::Quat::from_array(values));

                    state.rotations.get_mut(&id).unwrap().sync(*v);

                    true
                } else {
                    false
                }
            };

            if ui.combo_simple_string("##rotation_mode", &mut mode, &["Euler", "Quaternion"]) {
                state.rotations.get_mut(&id).unwrap().mode = mode;
            }

            if ui.is_item_hovered() {
                ui.tooltip_text("Euler angles use degrees and XYZ rotation order.");
            }

            changed
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

pub(super) fn edit_float_components<const N: usize>(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    name: &str,
    values: &mut [f32; N],
    prefixes: [&str; N],
    unit: &str,
) -> bool {
    properties::components(ui, values, prefixes, |prefix, value| {
        let label = format!("{name}:{prefix}");
        let format = format!("{}{unit}", float_format(*value));

        numeric_drag(
            ui,
            state,
            &label,
            value,
            if unit.is_empty() { 0.01 } else { 0.1 },
            &format,
            dear_imgui_rs::sys::ImGuiDataType_Float,
            None,
            None,
        )
    })
}

fn euler_degrees(quaternion: glam::Quat) -> [f32; 3] {
    let (x, y, z) = normalized_quaternion(quaternion).to_euler(glam::EulerRot::XYZ);

    [x, y, z].map(f32::to_degrees)
}

fn quaternion_from_degrees(angles: [f32; 3]) -> glam::Quat {
    let [x, y, z] = angles.map(f32::to_radians);

    normalized_quaternion(glam::Quat::from_euler(glam::EulerRot::XYZ, x, y, z))
}

pub(super) fn numeric_drag<T>(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    label: &str,
    value: &mut T,
    speed: f32,
    format: &str,
    data_type: dear_imgui_rs::sys::ImGuiDataType,
    min: Option<&T>,
    max: Option<&T>,
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
            min.map_or(ptr::null(), |value| value as *const T as *const c_void),
            max.map_or(ptr::null(), |value| value as *const T as *const c_void),
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

pub(super) fn float_format(value: f32) -> String {
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
    use super::{
        RotationEdit, State, edit_value, euler_degrees, float_format, quaternion_from_degrees,
    };
    use crate::core::TrackValue;

    #[test]
    fn compound_fields_fit_a_narrow_inspector_and_keep_rotation_on_mode_change() {
        let _context_lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();

        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        context.io_mut().set_display_size([320.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);

        let mut state = State::default();
        let mut vector = TrackValue::Vector2(glam::Vec2::new(10.0, 20.0));
        let rotation = quaternion_from_degrees([30.0, 40.0, 50.0]);
        let mut quaternion = TrackValue::Quaternion(rotation);

        for frame in 0..2 {
            let ui = context.frame();

            ui.window("Inspector layout test")
                .position([0.0, 0.0], dear_imgui_rs::Condition::Always)
                .size([220.0, 400.0], dear_imgui_rs::Condition::Always)
                .build(|| {
                    let width = ui.content_region_avail_width();

                    {
                        let _id = ui.push_id("vector");

                        assert!(
                            !edit_value(
                                ui,
                                &mut state,
                                "Position with a name too long for this narrow inspector",
                                &mut vector,
                                None,
                                true
                            )
                            .changed
                        );

                        if frame == 1 {
                            let size = ui.item_rect_size();

                            assert!(size[1] >= ui.frame_height() * 2.0);
                            assert!(size[1] < ui.frame_height() * 3.0);
                            assert!(size[0] <= width);

                            let _id = ui.push_id("without_reset");

                            assert!(
                                !edit_value(ui, &mut state, "Position", &mut vector, None, false)
                                    .changed
                            );

                            assert_eq!(ui.item_rect_size(), size);
                        }
                    }

                    let _id = ui.push_id("rotation");

                    assert!(
                        !edit_value(ui, &mut state, "Rotation", &mut quaternion, None, false)
                            .changed
                    );

                    if frame == 1 {
                        assert!(ui.item_rect_size()[1] >= ui.frame_height() * 5.0);
                    }
                });

            context.end_frame();

            for rotation in state.rotations.values_mut() {
                rotation.mode = 1;
            }
        }

        assert_eq!(vector, TrackValue::Vector2(glam::Vec2::new(10.0, 20.0)));
        assert_eq!(quaternion, TrackValue::Quaternion(rotation));
    }

    #[test]
    fn euler_edits_preserve_rotation_and_sync_external_changes() {
        let original = quaternion_from_degrees([25.0, -40.0, 110.0]);
        let restored = quaternion_from_degrees(euler_degrees(original));

        assert!(original.dot(restored).abs() > 1.0 - 1e-6);
        assert!((original.length() - 1.0).abs() < 1e-6);

        assert!(quaternion_from_degrees([0.0, 90.0, 0.0]).abs_diff_eq(
            glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            1e-6
        ));

        let mut edit = RotationEdit::new(original);

        edit.euler = [385.0, -40.0, 110.0];

        edit.sync(original);
        assert_eq!(edit.euler[0], 385.0);

        edit.mode = 1;

        let external = glam::Quat::from_rotation_x(0.5);

        edit.sync(external);
        assert_eq!(edit.mode, 1);
        assert!(external.dot(quaternion_from_degrees(edit.euler)).abs() > 1.0 - 1e-6);
    }

    #[test]
    fn float_format_keeps_only_required_decimal_places() {
        assert_eq!(float_format(1.0), "%.1f");
        assert_eq!(float_format(1.2323), "%.4f");
        assert_eq!(float_format(2.4300), "%.2f");
        assert_eq!(float_format(1e-20), "%.9g");
    }
}
