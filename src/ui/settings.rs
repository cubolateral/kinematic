use std::{fs, path::Path};

use crate::ui::inputs::{self, Category};
use crate::{core::ProjectSettings, editor::Editor};

use super::{properties::field, theme::Appearance, widgets::numeric_input_arrows};

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

        super::windows::window(ui, PROJECT_WINDOW, [520.0, 220.0])
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
                            |value| input_int_components(ui, value, ["Width", "Height"]),
                        );
                        let fps_changed =
                            field(ui, "FPS", &mut state.fps, defaults.fps as i32, |value| {
                                input_int(ui, "##value", value)
                            });
                        let margin_changed = field(
                            ui,
                            "Margin (px)",
                            &mut state.margin,
                            [defaults.margin.0 as i32, defaults.margin.1 as i32],
                            |value| input_int_components(ui, value, ["Width", "Height"]),
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

    super::windows::window(ui, EDITOR_WINDOW, [520.0, 420.0])
        .opened(&mut open)
        .build(|| {
            if let Some(_tabs) = ui.tab_bar("Editor settings sections") {
                if let Some(_editor) = ui.tab_item("Code Editor") {
                    let settings = &mut appearance.editor;
                    let defaults = super::code::Settings::default();

                    changed |= field(
                        ui,
                        "Font size",
                        &mut settings.font_size,
                        defaults.font_size,
                        |value| ui.slider_f32("##value", value, 10.0, 28.0),
                    );
                    changed |= field(
                        ui,
                        "Tab size",
                        &mut settings.tab_size,
                        defaults.tab_size,
                        |value| ui.slider("##value", 1usize, 8usize, value),
                    );
                    changed |= field(
                        ui,
                        "Insert spaces",
                        &mut settings.insert_spaces,
                        defaults.insert_spaces,
                        |value| ui.checkbox("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Word wrap",
                        &mut settings.word_wrap,
                        defaults.word_wrap,
                        |value| ui.checkbox("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Rulers",
                        &mut settings.rulers,
                        defaults.rulers,
                        |value| ui.input_text("##value", value).build(),
                    );
                    ui.text_disabled("Comma-separated columns, e.g. 80,100.");
                    changed |= field(
                        ui,
                        "Line numbers",
                        &mut settings.line_numbers,
                        defaults.line_numbers,
                        |value| ui.checkbox("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Automatic suggestions",
                        &mut settings.autocomplete,
                        defaults.autocomplete,
                        |value| ui.checkbox("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Show whitespace",
                        &mut settings.show_spaces,
                        defaults.show_spaces,
                        |value| ui.checkbox("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Hover delay (ms)",
                        &mut settings.hover_delay_ms,
                        defaults.hover_delay_ms,
                        |value| ui.slider("##value", 200usize, 2000usize, value),
                    );

                    ui.separator_with_text("Formatting");
                    changed |= field(
                        ui,
                        "Format on save",
                        &mut settings.format_on_save,
                        defaults.format_on_save,
                        |value| ui.checkbox("##value", value),
                    );
                    ui.text_disabled("Uses the tab size and indentation settings above.");
                    changed |= field(
                        ui,
                        "Trim trailing whitespace",
                        &mut settings.trim_trailing_whitespace,
                        defaults.trim_trailing_whitespace,
                        |value| ui.checkbox("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Insert final newline",
                        &mut settings.insert_final_newline,
                        defaults.insert_final_newline,
                        |value| ui.checkbox("##value", value),
                    );
                }

                if let Some(_shortcuts) = ui.tab_item("Shortcuts") {
                    draw_shortcuts(ui);
                }

                if let Some(_appearance) = ui.tab_item("Appearance") {
                    let defaults = Appearance::default();

                    changed |= field(
                        ui,
                        "Background",
                        &mut appearance.background,
                        defaults.background,
                        |value| ui.color_edit4("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Accent",
                        &mut appearance.accent,
                        defaults.accent,
                        |value| ui.color_edit4("##value", value),
                    );
                    changed |= field(
                        ui,
                        "Contrast",
                        &mut appearance.contrast,
                        defaults.contrast,
                        |value| ui.slider_f32("##value", value, 0.25, 1.0),
                    );
                    changed |= field(
                        ui,
                        "UI Scale",
                        &mut appearance.scale,
                        defaults.scale,
                        |value| ui.slider_f32("##value", value, 0.75, 1.25),
                    );
                }
            }
        });

    state.editor_open = open;

    changed
}

fn draw_shortcuts(ui: &dear_imgui_rs::Ui) {
    let flags = dear_imgui_rs::TableFlags::SCROLL_Y;

    let Some(_table) = ui.begin_table_with_sizing(
        "Shortcut list",
        2,
        dear_imgui_rs::TableOptions::from(flags),
        [0.0, ui.content_region_avail()[1]],
        0.0,
    ) else {
        return;
    };

    ui.table_setup_scroll_freeze(0, 1);
    ui.table_setup_column("Action", dear_imgui_rs::TableColumnFlags::empty(), None);
    ui.table_setup_column("Binding", dear_imgui_rs::TableColumnFlags::empty(), None);
    ui.table_headers_row();

    for (category, title) in [
        (Category::Editor, "Editor"),
        (Category::CodeEditor, "Code Editor"),
    ] {
        ui.table_next_row();
        ui.table_next_column();

        let group = ui
            .tree_node_config(title)
            .default_open(true)
            .open_on_arrow(true)
            .open_on_double_click(true)
            .no_tree_push_on_open(true)
            .push();

        ui.table_next_column();

        if group.is_some() {
            for binding in inputs::map()
                .bindings
                .iter()
                .filter(|binding| binding.category == category)
            {
                ui.table_next_row();
                ui.table_next_column();
                ui.set_cursor_pos_x(ui.cursor_pos_x() + ui.tree_node_to_label_spacing());
                ui.text(binding.name);
                ui.table_next_column();
                ui.text(&binding.display);
            }
        }
    }
}

fn input_int(ui: &dear_imgui_rs::Ui, label: &str, value: &mut i32) -> bool {
    let mut changed = ui.input_scalar(label, value).build();

    changed |= numeric_input_arrows(ui, value);

    changed
}

fn input_int_components<const N: usize>(
    ui: &dear_imgui_rs::Ui,
    values: &mut [i32; N],
    component_names: [&str; N],
) -> bool {
    super::properties::components(ui, values, component_names, |_, value| {
        input_int(ui, "##value", value)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_open_independently_and_start_centered() {
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
        assert!(!layout.contains("[Window][Editor settings]"));

        state.editor_open = true;

        assert!(!draw(&mut appearance, &mut state, None, context.frame()));
        context.end_frame();
        context.save_ini_settings(&mut layout);
        assert!(layout.contains("[Window][Editor settings]"));
        assert!(!layout.contains("[Window][Project settings]"));
        assert!(!state.initialized);

        for _ in 0..2 {
            assert!(!draw(&mut appearance, &mut state, None, context.frame()));
            context.end_frame();
        }

        layout.clear();
        context.save_ini_settings(&mut layout);
        assert!(layout.contains("[Window][Editor settings]\nPos=380,150\nSize=520,420"));

        state.editor_open = false;
        state.project_open = true;

        for _ in 0..2 {
            assert!(!draw(&mut appearance, &mut state, None, context.frame()));
            context.end_frame();
        }

        layout.clear();
        context.save_ini_settings(&mut layout);
        assert!(layout.contains("[Window][Project settings]\nPos=380,250\nSize=520,220"));
        assert!(!state.editor_open);

        state.editor_open = true;

        assert!(!draw(&mut appearance, &mut state, None, context.frame()));
        context.end_frame();
        assert!(state.project_open && state.editor_open);
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
