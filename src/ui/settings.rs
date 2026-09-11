use std::{fs, path::Path};

use crate::{core::ProjectSettings, editor::Editor};

use super::{
    theme::Appearance,
    widgets::{numeric_input_arrows, text_size},
};

pub(super) const WINDOW_NAME: &str = "Configuration";
const SETTINGS_PATH: &str = ".kinematic/settings.ron";

#[derive(Default)]
pub(super) struct State {
    initialized: bool,
    resolution: [i32; 2],
    fps: i32,
}

pub(super) fn load() -> Appearance {
    let path = Path::new(SETTINGS_PATH);

    match fs::read_to_string(path) {
        Ok(contents) => match ron::from_str(&contents) {
            Ok(appearance) => appearance,
            Err(error) => {
                eprintln!("Could not parse {SETTINGS_PATH}: {error}.");
                let appearance = Appearance::default();
                save(&appearance);
                appearance
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let appearance = Appearance::default();
            save(&appearance);
            appearance
        }
        Err(error) => {
            eprintln!("Could not read {SETTINGS_PATH}: {error}.");
            Appearance::default()
        }
    }
}

pub(super) fn save(appearance: &Appearance) {
    if let Err(error) = write(appearance, Path::new(SETTINGS_PATH)) {
        eprintln!("Could not write {SETTINGS_PATH}: {error}.");
    }
}

fn write(appearance: &Appearance, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }

    let contents = ron::ser::to_string_pretty(appearance, ron::ser::PrettyConfig::default())?;
    fs::write(path, format!("{contents}\n"))?;
    Ok(())
}

pub(super) fn draw(
    appearance: &mut Appearance,
    state: &mut State,
    editor: &mut Editor,
    ui: &dear_imgui_rs::Ui,
) -> bool {
    let mut changed = false;

    ui.window(WINDOW_NAME).build(|| {
        if !state.initialized {
            let (_, settings) = editor.get_project_info();
            state.resolution = [settings.resolution.0 as i32, settings.resolution.1 as i32];
            state.fps = settings.fps as i32;
            state.initialized = true;
        }

        ui.separator_with_text("Project");
        let project_changed = {
            let _disabled = ui.begin_disabled_with_cond(editor.is_exporting());
            let resolution_changed =
                input_int_components(ui, "Resolution", &mut state.resolution, ["Width", "Height"]);
            let fps_changed = input_int(ui, "FPS", &mut state.fps);
            resolution_changed || fps_changed
        };
        let valid_resolution = state
            .resolution
            .iter()
            .all(|value| *value > 0 && value % 2 == 0);
        let valid_fps = state.fps > 0;

        if !valid_resolution {
            ui.text_wrapped("Resolution dimensions must be positive, even numbers.");
        }
        if !valid_fps {
            ui.text_wrapped("FPS must be greater than zero.");
        }
        if project_changed && valid_resolution && valid_fps {
            let settings = ProjectSettings {
                resolution: (state.resolution[0] as u32, state.resolution[1] as u32),
                fps: state.fps as u32,
            };
            if editor.get_project_info().1 != settings {
                editor.request_project_settings(settings);
            }
        }

        ui.spacing();
        ui.separator_with_text("Appearance");

        changed |= ui.color_edit4("Background", &mut appearance.background);
        changed |= ui.color_edit4("Accent", &mut appearance.accent);
        changed |= ui.slider_f32("Contrast", &mut appearance.contrast, 0.25, 1.0);
        changed |= ui.slider_f32("UI Scale", &mut appearance.scale, 0.75, 1.25);

        ui.spacing();
        ui.separator();
        ui.spacing();

        if ui.button_with_size("Reset Appearance", [ui.content_region_avail_width(), 0.0]) {
            let defaults = Appearance::default();
            appearance.background = defaults.background;
            appearance.accent = defaults.accent;
            appearance.contrast = defaults.contrast;
            appearance.scale = defaults.scale;

            changed = true;
        }
    });

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
    fn settings_are_written_as_readable_ron() {
        let directory =
            std::env::temp_dir().join(format!("kinematic-settings-test-{}", std::process::id()));
        let path = directory.join("settings.ron");
        let appearance = Appearance {
            background: [0.1, 0.2, 0.3, 1.0],
            accent: [0.4, 0.5, 0.6, 1.0],
            contrast: 0.75,
            scale: 1.25,
        };

        write(&appearance, &path).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let loaded: Appearance = ron::from_str(&contents).unwrap();
        assert_eq!(loaded.background, appearance.background);
        assert_eq!(loaded.accent, appearance.accent);
        assert_eq!(loaded.contrast, appearance.contrast);
        assert_eq!(loaded.scale, appearance.scale);

        fs::remove_dir_all(directory).unwrap();
    }
}
