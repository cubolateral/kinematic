use kinematic_ui::widgets;
use std::{fs, path::Path};

use crate::ui::inputs::{self, Category};
use crate::{
    core::{Margin, MarginUnit, ProjectSettings},
    editor::Editor,
};

use super::{properties::field, theme::Appearance};
use kinematic_ui::{controls, numbers};

const PROJECT_WINDOW: &str = "Project settings";
const EDITOR_WINDOW: &str = "Editor settings";
const SETTINGS_PATH: &str = "settings.ron";

#[derive(Default)]
pub(super) struct State {
    pub(super) project_open: bool,
    pub(super) editor_open: bool,
    initialized: bool,
    name: String,
    pub(super) name_error: Option<String>,
    resolution: [i32; 2],
    fps: i32,
    margin: Margin,
    numeric: numbers::State,
}

pub(super) fn load() -> Appearance {
    let Ok(path) = crate::storage::global_path(SETTINGS_PATH) else {
        return Appearance::default();
    };

    fs::read_to_string(path)
        .ok()
        .and_then(|text| ron::from_str(&text).ok())
        .unwrap_or_default()
}

pub(super) fn save(appearance: &Appearance) {
    let Ok(path) = crate::storage::global_path(SETTINGS_PATH) else {
        return;
    };

    if path.exists()
        && fs::read_to_string(&path)
            .ok()
            .and_then(|text| ron::from_str::<Appearance>(&text).ok())
            .is_none()
    {
        return;
    }

    if let Err(error) = write(appearance, &path) {
        eprintln!("Could not save appearance: {error}.");
    }
}

fn write(appearance: &Appearance, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }

    let contents = ron::ser::to_string_pretty(appearance, ron::ser::PrettyConfig::default())?;
    let expected = fs::read(path).ok();

    crate::core::atomic_write(
        path,
        format!("{contents}\n").as_bytes(),
        expected.as_deref(),
    )?;

    Ok(())
}

pub(super) fn draw(
    appearance: &mut Appearance,
    state: &mut State,
    mut editor: Option<&mut Editor>,
    ui: &dear_imgui_rs::Ui,
) -> bool {
    let mut changed = false;

    if state.project_open {
        let mut open = true;

        super::windows::window(ui, PROJECT_WINDOW)
            .opened(&mut open)
            .build(|| {
                if let Some(editor) = editor.as_deref_mut() {
                    if !state.initialized {
                        let (name, settings) = editor.project_info();

                        state.name = name.into();
                        state.resolution =
                            [settings.resolution.0 as i32, settings.resolution.1 as i32];
                        state.fps = settings.fps as i32;
                        state.margin = settings.margin;
                        state.initialized = true;
                    }

                    let project_changed = {
                        let _disabled = ui.begin_disabled_with_cond(editor.is_exporting());
                        let edit = super::properties::row(ui, "Name", false, || {
                            controls::text_submit(ui, "##Name", &mut state.name)
                        });

                        if edit.changed || edit.committed {
                            state.name_error = editor.rename_project(&state.name).err();
                        }

                        let defaults = ProjectSettings::default();
                        let resolution_changed = field(
                            ui,
                            "Resolution",
                            &mut state.resolution,
                            [defaults.resolution.0 as i32, defaults.resolution.1 as i32],
                            |value| input_int_components(ui, &mut state.numeric, value),
                        );
                        let fps_changed =
                            field(ui, "FPS", &mut state.fps, defaults.fps as i32, |value| {
                                input_int(ui, &mut state.numeric, "##value", value)
                            });

                        super::properties::section(ui, "Margin");

                        let mut margin_changed = field(
                            ui,
                            "Unit",
                            &mut state.margin.unit,
                            defaults.margin.unit,
                            |value| {
                                let mut index = usize::from(*value == MarginUnit::Pixels);
                                let changed = controls::choice(
                                    ui,
                                    "##value",
                                    &mut index,
                                    &["Percentage (%)", "Pixels (px)"],
                                );

                                if changed {
                                    *value = if index == 0 {
                                        MarginUnit::Percentage
                                    } else {
                                        MarginUnit::Pixels
                                    };
                                }

                                changed
                            },
                        );

                        for (label, value) in [
                            ("Left", &mut state.margin.left),
                            ("Right", &mut state.margin.right),
                            ("Top", &mut state.margin.top),
                            ("Bottom", &mut state.margin.bottom),
                        ] {
                            margin_changed |= field(ui, label, value, 5.0, |value| {
                                numbers::drag(
                                    ui,
                                    &mut state.numeric,
                                    "##value",
                                    value,
                                    0.1,
                                    &numbers::float_format(*value),
                                    None,
                                    None,
                                )
                            });
                        }

                        resolution_changed || fps_changed || margin_changed
                    };
                    let valid_resolution = state
                        .resolution
                        .iter()
                        .all(|value| *value > 0 && *value <= 8192 && value % 2 == 0);
                    let valid_fps = state.fps > 0 && state.fps <= 1000;
                    let valid_margin = state.margin.is_valid((
                        state.resolution[0].max(0) as u32,
                        state.resolution[1].max(0) as u32,
                    ));

                    if !valid_resolution {
                        widgets::message(
                            ui,
                            "Resolution dimensions must be even numbers from 2 to 8192.",
                        );
                    }

                    if !valid_fps {
                        widgets::message(ui, "FPS must be between 1 and 1000.");
                    }

                    if !valid_margin {
                        widgets::message(ui, "Margin must fit inside the resolution.");
                    }

                    if project_changed && valid_resolution && valid_fps && valid_margin {
                        let settings = ProjectSettings {
                            resolution: (state.resolution[0] as u32, state.resolution[1] as u32),
                            fps: state.fps as u32,
                            margin: state.margin,
                        };

                        if editor.project_info().1 != settings {
                            editor.request_project_settings(settings);
                        }
                    }
                }
            });

        state.project_open = open;
    }

    if !state.editor_open {
        return false;
    }

    let mut open = true;

    super::windows::window(ui, EDITOR_WINDOW)
        .opened(&mut open)
        .build(|| {
            if let Some(_tabs) = super::windows::tabs(ui, "Editor settings sections") {
                if let Some(_appearance) = super::windows::tab(ui, "Appearance") {
                    let defaults = Appearance::default();

                    changed |= field(
                        ui,
                        "Background",
                        &mut appearance.background,
                        defaults.background,
                        |value| controls::color(ui, "##value", value),
                    );

                    changed |= field(
                        ui,
                        "Accent",
                        &mut appearance.accent,
                        defaults.accent,
                        |value| controls::color(ui, "##value", value),
                    );

                    changed |= field(
                        ui,
                        "Contrast",
                        &mut appearance.contrast,
                        defaults.contrast,
                        |value| input_float(ui, &mut state.numeric, value, 0.25, 1.0),
                    );

                    changed |= field(
                        ui,
                        "Border radius",
                        &mut appearance.border_radius,
                        defaults.border_radius,
                        |value| input_float(ui, &mut state.numeric, value, 0.0, 12.0),
                    );
                    changed |= field(
                        ui,
                        "Window spacing",
                        &mut appearance.window_spacing,
                        defaults.window_spacing,
                        |value| input_float(ui, &mut state.numeric, value, 1.0, 16.0),
                    );

                    changed |= field(
                        ui,
                        "UI Scale",
                        &mut appearance.scale,
                        defaults.scale,
                        |value| input_float(ui, &mut state.numeric, value, 0.75, 1.25),
                    );
                }

                if let Some(_shortcuts) = super::windows::tab(ui, "Shortcuts") {
                    draw_shortcuts(ui);
                }

                if let Some(_editor) = super::windows::tab(ui, "External editor") {
                    changed |= draw_external(&mut appearance.editor, ui);
                }
            }
        });

    state.editor_open = open;

    changed
}

fn draw_shortcuts(ui: &dear_imgui_rs::Ui) {
    let groups = [(Category::Editor, "Editor")].map(|(category, title)| {
        let bindings = inputs::map()
            .bindings
            .iter()
            .filter(|binding| binding.category == category)
            .map(|binding| (binding.name, binding.display.as_str()))
            .collect();

        (title, bindings)
    });

    kinematic_ui::lists::bindings(ui, groups);
}

fn input_int(
    ui: &dear_imgui_rs::Ui,
    state: &mut numbers::State,
    label: &str,
    value: &mut i32,
) -> bool {
    numbers::drag(ui, state, label, value, 1.0, "%d", None, None)
}

fn input_int_components(
    ui: &dear_imgui_rs::Ui,
    state: &mut numbers::State,
    values: &mut [i32; 2],
) -> bool {
    super::properties::components(ui, values, ["x", "y"], |_, value| {
        input_int(ui, state, "##value", value)
    })
}

fn input_float(
    ui: &dear_imgui_rs::Ui,
    state: &mut numbers::State,
    value: &mut f32,
    min: f32,
    max: f32,
) -> bool {
    let changed = numbers::drag(
        ui,
        state,
        "##value",
        value,
        0.01,
        &numbers::float_format(*value),
        Some(&min),
        Some(&max),
    );

    if changed {
        *value = if value.is_finite() {
            value.clamp(min, max)
        } else {
            min
        };
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_open_independently_without_saving_window_geometry() {
        let _context_lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);

        let mut state = State::default();
        let mut appearance = Appearance::default();

        state.editor_open = true;

        draw(&mut appearance, &mut state, None, context.frame());

        context.end_frame();

        assert!(state.editor_open && !state.project_open);

        state.project_open = true;

        draw(&mut appearance, &mut state, None, context.frame());

        context.end_frame();

        assert!(state.editor_open && state.project_open);

        let mut saved = String::new();

        context.save_ini_settings(&mut saved);

        assert!(!saved.contains("[Window][Editor settings]"));
        assert!(!saved.contains("[Window][Project settings]"));
    }

    #[test]
    fn older_settings_receive_editor_defaults() {
        let appearance: Appearance = ron::from_str("(background: (0.0, 0.0, 0.0, 1.0), accent: (0.0, 0.5, 1.0, 1.0), contrast: 1.0, scale: 1.0)").unwrap();

        assert_eq!(appearance.editor, super::super::code::Settings::default());
    }

    #[test]
    fn former_border_settings_use_the_default_corner_radius() {
        let appearance: Appearance =
            ron::from_str("(border_size: 2.0, window_spacing: 6.0)").unwrap();

        assert_eq!(appearance.border_radius, kinematic_ui::theme::CORNER_RADIUS);
        assert_eq!(appearance.window_spacing, 6.0);
    }

    #[test]
    fn embedded_editor_preferences_do_not_replace_external_defaults() {
        let appearance: Appearance = ron::from_str("(background: (0.1, 0.2, 0.3, 1.0), editor: (font_size: 18.0, colors: (theme: Custom, values: [])))").unwrap();
        assert_eq!(appearance.background, [0.1, 0.2, 0.3, 1.0]);
        assert_eq!(appearance.editor, super::super::code::Settings::default());
    }

    #[test]
    fn settings_are_written_as_readable_ron() {
        let directory =
            std::env::temp_dir().join(format!("kinematic-settings-test-{}", std::process::id()));
        let path = directory.join("settings.ron");
        let appearance = Appearance {
            background: [0.1, 0.2, 0.3, 1.0],
            accent: [0.4, 0.5, 0.6, 1.0],
            contrast: 0.75,
            scale: 1.25,
            border_radius: 2.0,
            window_spacing: 6.0,
            editor: super::super::code::Settings {
                executable: "codium".into(),
                ..Default::default()
            },
        };

        write(&appearance, &path).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let loaded: Appearance = ron::from_str(&contents).unwrap();

        assert_eq!(loaded.background, appearance.background);
        assert_eq!(loaded.accent, appearance.accent);
        assert_eq!(loaded.contrast, appearance.contrast);
        assert_eq!(loaded.border_radius, appearance.border_radius);
        assert_eq!(loaded.window_spacing, appearance.window_spacing);
        assert_eq!(loaded.scale, appearance.scale);
        assert_eq!(loaded.editor, appearance.editor);

        fs::remove_dir_all(directory).unwrap();
    }
}

fn draw_external(settings: &mut crate::editor::external::Settings, ui: &dear_imgui_rs::Ui) -> bool {
    use kinematic_ui::{controls, properties};

    let mut profile = settings.profile.min(2);
    let mut changed = properties::row(ui, "Profile", false, || {
        controls::choice(
            ui,
            "##Profile",
            &mut profile,
            &["VS Code", "VSCodium", "Custom graphical editor"],
        )
    })
    .changed;
    if changed {
        *settings = crate::editor::external::Settings::profile(profile);
    }
    for (label, value) in [
        ("Executable", &mut settings.executable),
        ("Project arguments", &mut settings.project_arguments),
        ("File arguments", &mut settings.file_arguments),
    ] {
        changed |= properties::row(ui, label, false, || {
            controls::text_input(ui, "##value", value)
        })
        .changed;
    }
    changed
}
