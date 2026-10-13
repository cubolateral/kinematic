use std::{fs, path::Path};

use crate::{core::ProjectSettings, editor::Editor};

use super::{
    theme::Appearance,
    widgets::{numeric_input_arrows, text_size},
};

pub(super) const WINDOW_NAME: &str = "Settings";
const SETTINGS_PATH: &str = "settings.ron";

#[derive(Default)]
pub(super) struct State {
    pub(super) open: bool,
    pub(super) editor_requested: bool,
    initialized: bool,
    resolution: [i32; 2],
    fps: i32,
    margin: [i32; 2],
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

    if !state.open {
        return false;
    }
    let mut open = state.open;
    ui.window(WINDOW_NAME)
        .opened(&mut open)
        .flags(dear_imgui_rs::WindowFlags::NO_DOCKING)
        .size([520.0, 420.0], dear_imgui_rs::Condition::FirstUseEver)
        .build(|| {
            if let Some(_tabs) = ui.tab_bar("Settings sections") {
                if let Some(_project) = ui.tab_item("Project") {
                    if let Some(editor) = editor {
                        if !state.initialized {
                            let (_, settings) = editor.project_info();
                            state.resolution =
                                [settings.resolution.0 as i32, settings.resolution.1 as i32];
                            state.fps = settings.fps as i32;
                            state.margin = [settings.margin.0 as i32, settings.margin.1 as i32];
                            state.initialized = true;
                        }

                        ui.separator_with_text("Project");
                        let project_changed = {
                            let _disabled = ui.begin_disabled_with_cond(editor.is_exporting());
                            let resolution_changed = input_int_components(
                                ui,
                                "Resolution",
                                &mut state.resolution,
                                ["Width", "Height"],
                            );
                            let fps_changed = input_int(ui, "FPS", &mut state.fps);
                            let margin_changed = input_int_components(
                                ui,
                                "Margin (px)",
                                &mut state.margin,
                                ["Width", "Height"],
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
                            ui.text_wrapped(
                                "Resolution dimensions must be even numbers from 2 to 8192.",
                            );
                        }
                        if !valid_fps {
                            ui.text_wrapped("FPS must be between 1 and 1000.");
                        }
                        if !valid_margin {
                            ui.text_wrapped("Margin must fit inside the resolution.");
                        }
                        if project_changed && valid_resolution && valid_fps && valid_margin {
                            let settings = ProjectSettings {
                                resolution: (
                                    state.resolution[0] as u32,
                                    state.resolution[1] as u32,
                                ),
                                fps: state.fps as u32,
                                margin: (state.margin[0] as u32, state.margin[1] as u32),
                            };
                            if editor.project_info().1 != settings {
                                editor.request_project_settings(settings);
                            }
                        }
                    }
                }
                let editor_flags = if state.editor_requested {
                    dear_imgui_rs::TabItemFlags::SET_SELECTED
                } else {
                    dear_imgui_rs::TabItemFlags::NONE
                };
                if let Some(_editor) = ui.tab_item_with_flags("Editor", None, editor_flags) {
                    state.editor_requested = false;
                    let settings = &mut appearance.editor;
                    changed |= ui.slider_f32("Font size", &mut settings.font_size, 10.0, 28.0);
                    changed |= ui.slider("Tab size", 1usize, 8usize, &mut settings.tab_size);
                    changed |= ui.checkbox("Insert spaces", &mut settings.insert_spaces);
                    changed |= ui.checkbox("Word wrap", &mut settings.word_wrap);
                    changed |= ui.checkbox("Line numbers", &mut settings.line_numbers);
                    changed |= ui.checkbox("Automatic suggestions", &mut settings.autocomplete);
                    changed |= ui.checkbox("Show whitespace", &mut settings.show_spaces);
                    changed |= ui.slider(
                        "Hover delay (ms)",
                        200usize,
                        2000usize,
                        &mut settings.hover_delay_ms,
                    );
                    ui.separator_with_text("Formatting");
                    changed |= ui.checkbox("Format on save", &mut settings.format_on_save);
                    ui.text_disabled("Uses the tab size and indentation settings above.");
                    changed |= ui.checkbox(
                        "Trim trailing whitespace",
                        &mut settings.trim_trailing_whitespace,
                    );
                    changed |=
                        ui.checkbox("Insert final newline", &mut settings.insert_final_newline);
                    if ui.button("Reset editor") {
                        *settings = super::code::Settings::default();
                        changed = true;
                    }
                }
                if let Some(_appearance) = ui.tab_item("Appearance") {
                    changed |= ui.color_edit4("Background", &mut appearance.background);
                    changed |= ui.color_edit4("Accent", &mut appearance.accent);
                    changed |= ui.slider_f32("Contrast", &mut appearance.contrast, 0.25, 1.0);
                    changed |= ui.slider_f32("UI Scale", &mut appearance.scale, 0.75, 1.25);

                    ui.spacing();
                    ui.separator();
                    ui.spacing();

                    if ui.button_with_size(
                        "Reset Appearance",
                        [ui.content_region_avail_width(), 0.0],
                    ) {
                        let defaults = Appearance::default();
                        appearance.background = defaults.background;
                        appearance.accent = defaults.accent;
                        appearance.contrast = defaults.contrast;
                        appearance.scale = defaults.scale;

                        changed = true;
                    }
                }
            }
        });

    state.open = open;
    changed
}

fn input_int(ui: &dear_imgui_rs::Ui, label: &str, value: &mut i32) -> bool {
    let mut changed = ui.input_scalar(label, value).build();
    changed |= numeric_input_arrows(ui, value);
    changed
}

fn input_int_components<const N: usize>(
    ui: &dear_imgui_rs::Ui,
    label: &str,
    values: &mut [i32; N],
    component_names: [&str; N],
) -> bool {
    let spacing = unsafe { ui.style().item_inner_spacing() }[0];
    let label_width = text_size(ui, label)[0];
    let fields_width = (ui.calc_item_width() - label_width - spacing).max(N as f32);
    let field_width = ((fields_width - spacing * N.saturating_sub(1) as f32) / N as f32).max(1.0);
    let mut changed = false;

    for (index, (value, component_name)) in values.iter_mut().zip(component_names).enumerate() {
        if index > 0 {
            ui.same_line_with_spacing(0.0, spacing);
        }

        ui.set_next_item_width(field_width);
        let component_label = format!("##{label}:{component_name}");
        changed |= input_int(ui, &component_label, value);
    }

    ui.same_line_with_spacing(0.0, spacing);
    ui.text(label);
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_open_only_on_request_and_work_without_a_project() {
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
        assert!(!draw(&mut appearance, &mut state, None, context.frame()));
        context.end_frame();
        let mut layout = String::new();
        context.save_ini_settings(&mut layout);
        assert!(!layout.contains("[Window][Settings]"));
        state.open = true;
        assert!(!draw(&mut appearance, &mut state, None, context.frame()));
        context.end_frame();
        context.save_ini_settings(&mut layout);
        assert!(layout.contains("[Window][Settings]"));
        assert!(!state.initialized);
    }

    #[test]
    fn older_settings_receive_editor_defaults() {
        let appearance: Appearance = ron::from_str("(background: (0.0, 0.0, 0.0, 1.0), accent: (0.0, 0.5, 1.0, 1.0), contrast: 1.0, scale: 1.0)").unwrap();
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
