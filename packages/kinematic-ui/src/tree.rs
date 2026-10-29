//! Hierarchy rows shared by editor trees; application objects stay with the caller.

use crate::widgets::text_size;
const ROW_HEIGHT: f32 = 24.0;
const INDENT: f32 = 16.0;
const NAME_SPACING: f32 = 4.0;

pub struct Row<'a> {
    pub label: &'a str,
    pub branches: &'a [bool],
    pub is_last: bool,
    pub expanded: bool,
    pub has_children: bool,
    pub highlighted: bool,
    pub visibility: Option<bool>,
    pub effective_visibility: bool,
}

#[derive(Default)]
pub struct Response {
    pub clicked: bool,
    pub toggled: bool,
    pub edit: bool,
    pub visibility: bool,
    pub hovered: bool,
}

pub fn row(ui: &dear_imgui_rs::Ui, row: Row<'_>) -> Response {
    let spacing = ui.clone_style().item_spacing();
    let _spacing = ui.push_style_var(dear_imgui_rs::StyleVar::ItemSpacing([spacing[0], 0.0]));
    let position = ui.cursor_screen_pos();
    let row_width = ui.content_region_avail_width();
    let visibility = row.visibility;
    let has_children = row.has_children;
    let row_id = "##row".to_owned();
    let control_count = 1.0 + f32::from(visibility.is_some());
    let selectable_width = (row_width - ROW_HEIGHT * control_count).max(1.0);
    let was_clicked = selectable_row(ui, row_id, [selectable_width, ROW_HEIGHT]);

    let hovered = ui.is_item_hovered();
    let text_y = position[1] + (ROW_HEIGHT - text_size(ui, row.label)[1]) * 0.5;
    let draw_list = ui.get_window_draw_list();
    let text_clip = draw_list.push_clip_rect(
        position,
        [position[0] + selectable_width, position[1] + ROW_HEIGHT],
        true,
    );

    let line_color = ui.get_color_u32(dear_imgui_rs::StyleColor::TreeLines);
    let middle = position[1] + ROW_HEIGHT * 0.5;

    for (depth, continues) in row.branches.iter().enumerate() {
        if *continues {
            let x = position[0] + INDENT * (depth as f32 + 0.5);

            draw_list
                .add_line([x, position[1]], [x, position[1] + ROW_HEIGHT], line_color)
                .build();
        }
    }

    let branch_x = position[0] + INDENT * (row.branches.len() as f32 + 0.5);
    let arrow_x = branch_x + INDENT * 0.5;

    draw_list
        .add_line(
            [branch_x, position[1]],
            [
                branch_x,
                if row.is_last {
                    middle
                } else {
                    position[1] + ROW_HEIGHT
                },
            ],
            line_color,
        )
        .build();

    draw_list
        .add_line([branch_x, middle], [arrow_x, middle], line_color)
        .build();

    if has_children && row.expanded {
        let x = arrow_x + INDENT * 0.5;

        draw_list
            .add_line([x, middle + 6.0], [x, position[1] + ROW_HEIGHT], line_color)
            .build();
    }

    draw_list.add_text(
        [arrow_x + INDENT + NAME_SPACING, text_y],
        ui.get_color_u32(if row.highlighted {
            dear_imgui_rs::StyleColor::CheckMark
        } else {
            dear_imgui_rs::StyleColor::Text
        }),
        row.label,
    );

    drop(text_clip);

    drop(draw_list);

    let toggled = has_children
        && draw_arrow(
            ui,
            [arrow_x, position[1]],
            row.expanded,
            was_clicked && ui.io().mouse_pos()[0] < position[0] + selectable_width,
        );

    ui.get_window_draw_list()
        .add_rect(
            [position[0] + selectable_width, position[1]],
            [position[0] + row_width, position[1] + ROW_HEIGHT],
            ui.get_color_u32(dear_imgui_rs::StyleColor::WindowBg),
        )
        .filled(true)
        .build();

    ui.same_line_with_spacing(0.0, 0.0);

    let edit = crate::controls::text_button(ui, crate::icons::PENCIL, [ROW_HEIGHT; 2]);
    if ui.is_item_hovered() {
        ui.tooltip_text("Edit in the matching canvas.");
    }
    let mut visibility_clicked = false;
    if let Some(visible) = visibility {
        ui.same_line_with_spacing(0.0, 0.0);
        visibility_clicked = crate::controls::text_button_colored(
            ui,
            if visible {
                crate::icons::EYE
            } else {
                crate::icons::EYE_SLASH
            },
            [ROW_HEIGHT; 2],
            if row.effective_visibility {
                dear_imgui_rs::StyleColor::Text
            } else {
                dear_imgui_rs::StyleColor::TextDisabled
            },
        );
        if ui.is_item_hovered() {
            ui.tooltip_text(if visible { "Hide." } else { "Show." });
        }
    }
    Response {
        clicked: was_clicked && !toggled,
        toggled,
        edit,
        visibility: visibility_clicked,
        hovered,
    }
}

fn draw_arrow(ui: &dear_imgui_rs::Ui, position: [f32; 2], expanded: bool, clicked: bool) -> bool {
    let center = [position[0] + INDENT * 0.5, position[1] + ROW_HEIGHT * 0.5];
    let (a, b, c) = if expanded {
        (
            [center[0] - 4.0, center[1] - 2.0],
            [center[0] + 4.0, center[1] - 2.0],
            [center[0], center[1] + 3.0],
        )
    } else {
        (
            [center[0] - 2.0, center[1] - 4.0],
            [center[0] + 3.0, center[1]],
            [center[0] - 2.0, center[1] + 4.0],
        )
    };

    ui.get_window_draw_list()
        .add_triangle(a, b, c, ui.get_color_u32(dear_imgui_rs::StyleColor::Text))
        .filled(true)
        .build();

    let mouse_x = ui.io().mouse_pos()[0];

    clicked && mouse_x >= position[0] && mouse_x < position[0] + INDENT
}

fn selectable_row(ui: &dear_imgui_rs::Ui, id: String, size: [f32; 2]) -> bool {
    let transparent = [0.0; 4];
    let _header = ui.push_style_color(dear_imgui_rs::StyleColor::Header, transparent);
    let _header_hovered =
        ui.push_style_color(dear_imgui_rs::StyleColor::HeaderHovered, transparent);
    let _header_active = ui.push_style_color(dear_imgui_rs::StyleColor::HeaderActive, transparent);

    ui.selectable_config(id).selected(false).size(size).build()
}

pub fn root(
    ui: &dear_imgui_rs::Ui,
    label: &str,
    expanded: bool,
    has_children: bool,
    selected: bool,
) -> Response {
    let spacing = ui.clone_style().item_spacing();
    let _spacing = ui.push_style_var(dear_imgui_rs::StyleVar::ItemSpacing([spacing[0], 0.0]));
    let position = ui.cursor_screen_pos();
    let root_clicked = selectable_row(ui, "##root".to_owned(), [0.0, ROW_HEIGHT]);
    let draw_list = ui.get_window_draw_list();

    draw_list.add_text(
        [
            position[0] + INDENT + NAME_SPACING,
            position[1] + (ROW_HEIGHT - text_size(ui, label)[1]) * 0.5,
        ],
        ui.get_color_u32(if selected {
            dear_imgui_rs::StyleColor::CheckMark
        } else {
            dear_imgui_rs::StyleColor::Text
        }),
        label,
    );

    if expanded && has_children {
        let x = position[0] + INDENT * 0.5;

        draw_list
            .add_line(
                [x, position[1] + ROW_HEIGHT * 0.5 + 6.0],
                [x, position[1] + ROW_HEIGHT],
                ui.get_color_u32(dear_imgui_rs::StyleColor::TreeLines),
            )
            .build();
    }

    drop(draw_list);

    let toggled = draw_arrow(ui, position, expanded, root_clicked);
    Response {
        clicked: root_clicked && !toggled,
        toggled,
        ..Response::default()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn arrows_selection_and_actions_are_separate_click_targets() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();
        context.io_mut().set_display_size([640.0, 480.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        let mut targets = [[0.0; 2]; 4];
        let mut actions = [[false; 4]; 4];
        for frame in 0..14 {
            if frame >= 2 {
                let target = (frame - 2) / 3;
                let phase = (frame - 2) % 3;
                context.io_mut().add_mouse_pos_event(targets[target]);
                if phase == 1 || phase == 2 {
                    context
                        .io_mut()
                        .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, phase == 1);
                }
            }
            let ui = context.frame();
            ui.window("Tree interaction test")
                .position([0.0; 2], dear_imgui_rs::Condition::Always)
                .size([360.0, 200.0], dear_imgui_rs::Condition::Always)
                .build(|| {
                    let min = ui.cursor_screen_pos();
                    let width = ui.content_region_avail_width();
                    targets = [
                        24.0,
                        80.0,
                        width - super::ROW_HEIGHT * 1.5,
                        width - super::ROW_HEIGHT * 0.5,
                    ]
                    .map(|x| [min[0] + x, min[1] + super::ROW_HEIGHT * 0.5]);
                    let response = super::row(
                        ui,
                        super::Row {
                            label: "Object",
                            branches: &[],
                            is_last: true,
                            expanded: true,
                            has_children: true,
                            highlighted: false,
                            visibility: Some(true),
                            effective_visibility: true,
                        },
                    );
                    if frame >= 2 {
                        for (output, value) in actions[(frame - 2) / 3].iter_mut().zip([
                            response.clicked,
                            response.toggled,
                            response.edit,
                            response.visibility,
                        ]) {
                            *output |= value;
                        }
                    }
                });
            context.end_frame();
        }
        assert_eq!(
            actions,
            [
                [false, true, false, false],
                [true, false, false, false],
                [false, false, true, false],
                [false, false, false, true]
            ]
        );
    }
}
