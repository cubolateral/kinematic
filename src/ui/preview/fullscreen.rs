use crate::{editor::Editor, ui::timeline};

use super::image;

const CONTROLS_HEIGHT: f32 = 72.0;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    let viewport = ui.main_viewport();
    let viewport_pos = viewport.pos();
    let viewport_size = viewport.size();
    let preview = image::preview(editor);
    let window_flags = dear_imgui_rs::WindowFlags::NO_DECORATION
        | dear_imgui_rs::WindowFlags::NO_MOVE
        | dear_imgui_rs::WindowFlags::NO_DOCKING
        | dear_imgui_rs::WindowFlags::NO_SAVED_SETTINGS
        | dear_imgui_rs::WindowFlags::NO_NAV_FOCUS;

    {
        let _padding = ui.push_style_var(dear_imgui_rs::StyleVar::WindowPadding([0.0; 2]));
        let _window_border = ui.push_style_var(dear_imgui_rs::StyleVar::WindowBorderSize(0.0));
        let _image_border = ui.push_style_var(dear_imgui_rs::StyleVar::ImageBorderSize(0.0));
        let _background =
            ui.push_style_color(dear_imgui_rs::StyleColor::WindowBg, [0.0, 0.0, 0.0, 1.0]);

        ui.window("Fullscreen preview.")
            .position(viewport_pos, dear_imgui_rs::Condition::Always)
            .size(viewport_size, dear_imgui_rs::Condition::Always)
            .flags(window_flags | dear_imgui_rs::WindowFlags::NO_INPUTS)
            .build(|| {
                image::draw(ui, preview, viewport_size);
            });
    }

    let is_exporting = editor.is_exporting();
    let fps = editor.get_preview_fps();

    timeline::shortcuts(editor.get_timeline(), ui, !is_exporting);

    let is_controlling = editor.get_timeline().is_controlling;
    if !controls_visible(
        ui.io().mouse_pos(),
        viewport_pos,
        viewport_size,
        is_controlling,
    ) {
        editor.get_timeline().is_controlling = false;
        return false;
    }

    let controls_pos = [
        viewport_pos[0],
        viewport_pos[1] + (viewport_size[1] - CONTROLS_HEIGHT).max(0.0),
    ];
    let controls_size = [viewport_size[0], CONTROLS_HEIGHT.min(viewport_size[1])];
    let mut toggle_fullscreen = false;

    let _text = ui.push_style_color(dear_imgui_rs::StyleColor::Text, [1.0, 1.0, 1.0, 1.0]);
    let _button = ui.push_style_color(dear_imgui_rs::StyleColor::Button, [0.12, 0.12, 0.12, 1.0]);
    let _button_hovered = ui.push_style_color(
        dear_imgui_rs::StyleColor::ButtonHovered,
        [0.22, 0.22, 0.22, 1.0],
    );
    let _button_active = ui.push_style_color(
        dear_imgui_rs::StyleColor::ButtonActive,
        [0.32, 0.32, 0.32, 1.0],
    );
    let _scrubber =
        ui.push_style_color(dear_imgui_rs::StyleColor::FrameBg, [0.35, 0.35, 0.35, 1.0]);

    ui.window("Fullscreen controls.")
        .position(controls_pos, dear_imgui_rs::Condition::Always)
        .size(controls_size, dear_imgui_rs::Condition::Always)
        .flags(window_flags)
        .build(|| {
            toggle_fullscreen =
                timeline::fullscreen_controls(editor.get_timeline(), ui, fps, !is_exporting);
        });

    toggle_fullscreen
}

fn controls_visible(
    mouse: [f32; 2],
    viewport_pos: [f32; 2],
    viewport_size: [f32; 2],
    is_controlling: bool,
) -> bool {
    if is_controlling {
        return true;
    }

    let right = viewport_pos[0] + viewport_size[0];
    let bottom = viewport_pos[1] + viewport_size[1];
    let reveal_height = CONTROLS_HEIGHT.min(viewport_size[1]);

    (viewport_pos[0]..=right).contains(&mouse[0])
        && (bottom - reveal_height..=bottom).contains(&mouse[1])
}

#[cfg(test)]
mod tests {
    use super::controls_visible;

    #[test]
    fn controls_are_revealed_only_at_the_viewport_bottom() {
        let position = [100.0, 50.0];
        let size = [800.0, 600.0];

        assert!(!controls_visible([500.0, 500.0], position, size, false));
        assert!(controls_visible([500.0, 640.0], position, size, false));
        assert!(!controls_visible([50.0, 640.0], position, size, false));
    }

    #[test]
    fn active_controls_remain_visible() {
        assert!(controls_visible(
            [0.0, 0.0],
            [100.0, 50.0],
            [800.0, 600.0],
            true,
        ));
    }
}
