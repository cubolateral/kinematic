use crate::{
    core::types::{Vector2, vec2},
    editor::Editor,
};

use super::timeline;

const FULLSCREEN_CONTROLS_HEIGHT: f32 = 72.0;
const VIEWPORT_OUTLINE_THICKNESS: f32 = 1.0;

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    let is_exporting = editor.is_exporting();
    let mut clicked = None;
    let (name, resolution, fps) = {
        let project = editor.get_project();
        (project.name, project.resolution, project.fps)
    };
    let (source_size, texture) = preview_image(editor);

    ui.set_next_window_class(
        &dear_imgui_rs::WindowClass::default()
            .dock_node_flags_override_set(dear_imgui_rs::DockFlags::AUTO_HIDE_TAB_BAR),
    );

    ui.window("Preview").build(|| {
        let _disabled = ui.begin_disabled_with_cond(is_exporting);
        ui.text(format!(
            "[PROJECT INFO] Name: {name} / Resolution: {}x{} / FPS: {fps}",
            resolution.0, resolution.1,
        ));
        ui.separator();

        clicked = draw_preview_image(ui, texture, source_size, ui.content_region_avail(), true);
    });

    if let Some(point) = clicked {
        editor.select_at(point);
    }
}

pub(super) fn draw_fullscreen(editor: &mut Editor, ui: &dear_imgui_rs::Ui) -> bool {
    let viewport = ui.main_viewport();
    let viewport_pos = viewport.pos();
    let viewport_size = viewport.size();
    let (source_size, texture) = preview_image(editor);
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
                draw_preview_image(ui, texture, source_size, viewport_size, false);
            });
    }

    let is_exporting = editor.is_exporting();
    let fps = editor.get_preview_fps();

    timeline::shortcuts(editor.get_timeline(), ui, !is_exporting);

    let is_controlling = editor.get_timeline().is_controlling;
    if !fullscreen_controls_visible(
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
        viewport_pos[1] + (viewport_size[1] - FULLSCREEN_CONTROLS_HEIGHT).max(0.0),
    ];
    let controls_size = [
        viewport_size[0],
        FULLSCREEN_CONTROLS_HEIGHT.min(viewport_size[1]),
    ];
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

fn preview_image(editor: &mut Editor) -> ([f32; 2], dear_imgui_rs::TextureId) {
    let preview = editor.get_preview();
    let (width, height) = preview.get_size();

    (
        [width.max(1) as f32, height.max(1) as f32],
        preview.get_imgui_texture_id(),
    )
}

fn draw_preview_image(
    ui: &dear_imgui_rs::Ui,
    texture: dear_imgui_rs::TextureId,
    source_size: [f32; 2],
    available: [f32; 2],
    interactive: bool,
) -> Option<Vector2> {
    let scale =
        (available[0].max(1.0) / source_size[0]).min(available[1].max(1.0) / source_size[1]);
    let size = [source_size[0] * scale, source_size[1] * scale];
    let origin = ui.cursor_screen_pos();

    ui.set_cursor_screen_pos([
        origin[0] + (available[0] - size[0]) * 0.5,
        origin[1] + (available[1] - size[1]) * 0.5,
    ]);
    ui.image_config(texture, size)
        .uv0([0.0, 1.0])
        .uv1([1.0, 0.0])
        .build();

    if !interactive {
        return None;
    }

    let image_min = ui.item_rect_min();
    let image_max = ui.item_rect_max();
    let clicked = if ui.is_item_hovered() && ui.is_mouse_clicked(dear_imgui_rs::MouseButton::Left) {
        let mouse = ui.io().mouse_pos();
        let x = (mouse[0] - image_min[0]) / size[0] * source_size[0] - source_size[0] * 0.5;
        let y = (mouse[1] - image_min[1]) / size[1] * source_size[1] - source_size[1] * 0.5;

        Some(vec2(x, y))
    } else {
        None
    };
    let half = VIEWPORT_OUTLINE_THICKNESS * 0.5;
    let min = [image_min[0] - half, image_min[1] - half];
    let max = [image_max[0] + half, image_max[1] + half];

    ui.get_window_draw_list()
        .add_rect(
            min,
            max,
            ui.get_color_u32(dear_imgui_rs::StyleColor::Border),
        )
        .thickness(VIEWPORT_OUTLINE_THICKNESS)
        .build();

    clicked
}

fn fullscreen_controls_visible(
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
    let reveal_height = FULLSCREEN_CONTROLS_HEIGHT.min(viewport_size[1]);

    (viewport_pos[0]..=right).contains(&mouse[0])
        && (bottom - reveal_height..=bottom).contains(&mouse[1])
}
