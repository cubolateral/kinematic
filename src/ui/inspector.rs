mod audio;

use crate::{
    core::{
        TrackChoices, TrackInfo, TrackLimits, TrackTarget, TrackValue,
        components::{Inspection, Name},
        normalized_quaternion,
        objects::{CanvasSettings, ProjectionSource},
    },
    editor::Editor,
};
use kinematic_ui::widgets;
use std::collections::HashMap;

use kinematic_ui::{controls, numbers, properties, windows};

pub(super) const WINDOW_NAME: &str = "Inspector";
#[derive(Default)]
pub(super) struct State {
    audio: audio::State,
    numeric: numbers::State,
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

    windows::panel(ui, WINDOW_NAME, || {
        super::timeline::audio_shortcuts(editor, ui, true);
        if let Some(clip) = editor.selected_audio() {
            audio::draw(editor, ui, &mut state.audio, clip);

            return;
        }
        state.audio = audio::State::default();

        let Some((scene_index, entity)) = selected else {
            widgets::muted(ui, "Nothing to inspect.");

            return;
        };

        let _object_id = ui.push_id(&format!("{scene_index}:{}", entity.to_bits()));

        let (scene, inspector_edits) = editor.inspector_scene_mut(scene_index);
        let world = scene.world();

        let Ok(inspection) = world.get::<&Inspection>(entity) else {
            widgets::muted(ui, "The selected object is unavailable.");

            return;
        };

        let name = world
            .get::<&Name>(entity)
            .expect("Inspected object must contain a Name component.");
        header(ui, name.get(), &format!("Type: {}", inspection.object_name));

        if let Ok(settings) = world.get::<&CanvasSettings>(entity) {
            properties::read_only(
                ui,
                "Resolution",
                &format!("{} x {}.", settings.resolution.0, settings.resolution.1),
            );
        }
        if let Ok(source) = world.get::<&ProjectionSource>(entity) {
            properties::read_only(
                ui,
                "Source canvas",
                &source.0.map_or_else(
                    || "Unassigned.".into(),
                    |texture| format!("{}.", texture.entity.to_bits()),
                ),
            );
        }

        let _disabled = ui.begin_disabled_with_cond(is_exporting || is_playing);

        for (index, target) in inspection
            .properties(&world, entity)
            .into_iter()
            .filter(|target| {
                inspection.object_name != "Code"
                    || !matches!(target.name(), "selection" | "highlights")
            })
            .enumerate()
        {
            let _id = ui.push_id(index);
            let mut value = target.get(&world, entity);
            let before = value.clone();
            let track = match &target {
                TrackTarget::Property { info, .. } => Some(*info),
                _ => None,
            };
            let edit = edit_value(
                ui,
                state,
                target.name(),
                &mut value,
                track,
                inspector_edits.contains(entity, &target),
            );

            if edit.changed {
                value = target.clamp(value);
                let valid = match &target {
                    TrackTarget::Dynamic(schema) => schema.validate(&value).is_ok(),
                    _ => value.validate().is_ok(),
                };
                if valid && value != before {
                    inspector_edits.record(entity, target.clone(), before);
                    target.set(&world, entity, value);
                    scene.invalidate();
                }
            }
            if edit.reset {
                inspector_edits.reset(scene, entity, &target);
            }
        }
    });
}

fn header(ui: &dear_imgui_rs::Ui, name: &str, details: &str) {
    widgets::text(ui, name);
    ui.same_line();
    widgets::muted(ui, details);
    widgets::separator(ui);
    ui.spacing();
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
        TrackValue::Bool(v) => {
            controls::checkbox(ui, if *v { "On###value" } else { "Off###value" }, v)
        }
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

            if controls::choice(ui, name, &mut current, variants) {
                *v = variants[current];

                true
            } else {
                false
            }
        }
        TrackValue::F32(v) => {
            let format = numbers::float_format(*v);
            let (min, max) = match track.map(|track| track.limits) {
                Some(TrackLimits::F32 { min, max }) => (min, max),
                _ => (None, None),
            };

            numbers::drag(
                ui,
                &mut state.numeric,
                name,
                v,
                0.01,
                &format,
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
        TrackValue::U32(v) => numbers::drag(
            ui,
            &mut state.numeric,
            name,
            v,
            1.0,
            "%u",
            match track.map(|track| &track.limits) {
                Some(TrackLimits::U32 { min, .. }) => min.as_ref(),
                _ => None,
            },
            match track.map(|track| &track.limits) {
                Some(TrackLimits::U32 { max, .. }) => max.as_ref(),
                _ => None,
            },
        ),
        TrackValue::I32(v) => numbers::drag(
            ui,
            &mut state.numeric,
            name,
            v,
            1.0,
            "%d",
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

            if controls::choice(ui, "##rotation_mode", &mut mode, &["Euler", "Quaternion"]) {
                state.rotations.get_mut(&id).unwrap().mode = mode;
            }

            if ui.is_item_hovered() {
                widgets::tooltip(ui, "Euler angles use degrees and XYZ rotation order");
            }

            changed
        }
        TrackValue::Color(v) => {
            let mut values = v.rgba();

            if controls::color(ui, name, &mut values) {
                *v = crate::core::types::Color::new(values[0], values[1], values[2], values[3]);

                true
            } else {
                false
            }
        }
        TrackValue::CodeRanges(v) => {
            widgets::text(ui, format!("{name}: {v}"));
            false
        }
        TrackValue::String(v) => controls::text_area(ui, name, v),
    }
}

fn edit_float_components<const N: usize>(
    ui: &dear_imgui_rs::Ui,
    state: &mut State,
    name: &str,
    values: &mut [f32; N],
    prefixes: [&str; N],
    unit: &str,
) -> bool {
    properties::components(ui, values, prefixes, |prefix, value| {
        let label = format!("{name}:{prefix}");
        let format = format!("{}{unit}", numbers::float_format(*value));

        numbers::drag(
            ui,
            &mut state.numeric,
            &label,
            value,
            if unit.is_empty() { 0.01 } else { 0.1 },
            &format,
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

#[cfg(test)]
mod tests {
    use super::{RotationEdit, State, edit_value, euler_degrees, quaternion_from_degrees};
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
}
