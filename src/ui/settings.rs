use kinematic_ui::widgets;
use std::{fs, path::Path};

use crate::ui::inputs::{self, Category};
use crate::{core::ProjectSettings, editor::Editor};

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
    resolution: [i32; 2],
    fps: i32,
    margin: [i32; 2],
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
    editor: Option<&mut Editor>,
    ui: &dear_imgui_rs::Ui,
) -> bool {
    let mut changed = false;

    if state.project_open {
        let mut open = true;

        super::windows::window(ui, PROJECT_WINDOW)
            .opened(&mut open)
            .build(|| {
                if let Some(editor) = editor {
                    if !state.initialized {
                        let (_, settings) = editor.project_info();

                        state.resolution =
                            [settings.resolution.0 as i32, settings.resolution.1 as i32];

                        state.fps = settings.fps as i32;
                        state.margin = [settings.margin.0 as i32, settings.margin.1 as i32];
                        state.initialized = true;
                    }

                    let project_changed = {
                        let _disabled = ui.begin_disabled_with_cond(editor.is_exporting());
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
                        let margin_changed = field(
                            ui,
                            "Margin (px)",
                            &mut state.margin,
                            [defaults.margin.0 as i32, defaults.margin.1 as i32],
                            |value| input_int_components(ui, &mut state.numeric, value),
                        );

                        resolution_changed || fps_changed || margin_changed
                    };
                    let valid_resolution = state
                        .resolution
                        .iter()
                        .all(|value| *value > 0 && *value <= 8192 && value % 2 == 0);
                    let valid_fps = state.fps > 0 && state.fps <= 1000;
                    let valid_margin = state.margin[0] >= 0
                        && state.margin[1] >= 0
                        && state.margin[0] < state.resolution[0] / 2
                        && state.margin[1] < state.resolution[1] / 2;

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
                            margin: (state.margin[0] as u32, state.margin[1] as u32),
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
                if let Some(_editor) = super::windows::tab(ui, "Code Editor") {
                    let settings = &mut appearance.editor;
                    let defaults = super::code::Settings::default();

                    changed |= field(
                        ui,
                        "Font size",
                        &mut settings.font_size,
                        defaults.font_size,
                        |value| input_float(ui, &mut state.numeric, value, 10.0, 28.0),
                    );
                    changed |= field(
                        ui,
                        "Line height",
                        &mut settings.line_height,
                        defaults.line_height,
                        |value| input_float(ui, &mut state.numeric, value, 1.0, 2.0),
                    );
                    changed |= field(
                        ui,
                        "Tab size",
                        &mut settings.tab_size,
                        defaults.tab_size,
                        |value| input_usize(ui, &mut state.numeric, value, 1, 8),
                    );
                    changed |= field(
                        ui,
                        "Insert spaces",
                        &mut settings.insert_spaces,
                        defaults.insert_spaces,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    changed |= field(
                        ui,
                        "Word wrap",
                        &mut settings.word_wrap,
                        defaults.word_wrap,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    changed |= field(
                        ui,
                        "Rulers",
                        &mut settings.rulers,
                        defaults.rulers,
                        |value| controls::text_input(ui, "##value", value),
                    );
                    widgets::muted(ui, "Comma-separated columns, e.g. 80,100.");
                    changed |= field(
                        ui,
                        "Line numbers",
                        &mut settings.line_numbers,
                        defaults.line_numbers,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    changed |= field(
                        ui,
                        "Automatic suggestions",
                        &mut settings.autocomplete,
                        defaults.autocomplete,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    changed |= field(
                        ui,
                        "Show whitespace",
                        &mut settings.show_spaces,
                        defaults.show_spaces,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    changed |= field(
                        ui,
                        "Hover delay (ms)",
                        &mut settings.hover_delay_ms,
                        defaults.hover_delay_ms,
                        |value| input_usize(ui, &mut state.numeric, value, 200, 2000),
                    );

                    super::properties::section(ui, "Files");
                    changed |= field(
                        ui,
                        "Auto save",
                        &mut settings.auto_save,
                        defaults.auto_save,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    changed |= field(
                        ui,
                        "Auto save delay (ms)",
                        &mut settings.auto_save_delay_ms,
                        defaults.auto_save_delay_ms,
                        |value| input_usize(ui, &mut state.numeric, value, 100, 60_000),
                    );

                    super::properties::section(ui, "Formatting");
                    changed |= field(
                        ui,
                        "Format on save",
                        &mut settings.format_on_save,
                        defaults.format_on_save,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    widgets::muted(ui, "Uses the tab size and indentation settings above.");
                    changed |= field(
                        ui,
                        "Trim trailing whitespace",
                        &mut settings.trim_trailing_whitespace,
                        defaults.trim_trailing_whitespace,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                    changed |= field(
                        ui,
                        "Insert final newline",
                        &mut settings.insert_final_newline,
                        defaults.insert_final_newline,
                        |value| controls::checkbox(ui, "##value", value),
                    );
                }

                if let Some(_shortcuts) = super::windows::tab(ui, "Shortcuts") {
                    draw_shortcuts(ui);
                }

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
                        "UI Scale",
                        &mut appearance.scale,
                        defaults.scale,
                        |value| input_float(ui, &mut state.numeric, value, 0.75, 1.25),
                    );
                    changed |= appearance.editor.colors.draw(ui);
                }
            }
        });

    state.editor_open = open;

    changed
}

fn draw_shortcuts(ui: &dear_imgui_rs::Ui) {
    let groups = [
        (Category::Editor, "Editor"),
        (Category::CodeEditor, "Code Editor"),
    ]
    .map(|(category, title)| {
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

fn input_usize(
    ui: &dear_imgui_rs::Ui,
    state: &mut numbers::State,
    value: &mut usize,
    min: i32,
    max: i32,
) -> bool {
    let mut number = (*value).min(i32::MAX as usize) as i32;
    let changed = numbers::drag(
        ui,
        state,
        "##value",
        &mut number,
        1.0,
        "%d",
        Some(&min),
        Some(&max),
    );
    if changed {
        *value = number.clamp(min, max) as usize;
    }
    changed
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
        assert_eq!(appearance.editor.line_height, 1.5);
        assert!(!appearance.editor.auto_save);
        assert_eq!(appearance.editor.auto_save_delay_ms, 100);
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
            editor: super::super::code::Settings {
                tab_size: 2,
                word_wrap: true,
                ..Default::default()
            },
        };

        write(&appearance, &path).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let loaded: Appearance = ron::from_str(&contents).unwrap();

        assert_eq!(loaded.background, appearance.background);
        assert_eq!(loaded.accent, appearance.accent);
        assert_eq!(loaded.contrast, appearance.contrast);
        assert_eq!(loaded.scale, appearance.scale);
        assert_eq!(loaded.editor, appearance.editor);

        fs::remove_dir_all(directory).unwrap();
    }
}
