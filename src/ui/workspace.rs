use super::{code, inspector, preview, scene_tree, timeline};
use dear_imgui_rs::{Condition, DockLayout, DockLayoutApply, DockNodeFlags, StyleVar, WindowFlags};
use serde::{Deserialize, Serialize};

#[cfg(test)]
const GAP: f32 = 4.0;
const PANEL_NAMES: [&str; 3] = [
    code::FILES_WINDOW,
    scene_tree::WINDOW_NAME,
    inspector::WINDOW_NAME,
];

pub(super) const BOTTOM_PANELS: [&str; 2] = [timeline::WINDOW_NAME, code::CONSOLE_WINDOW];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(super) struct Layout {
    positions: [usize; 3],
    widths: [f32; 2],
    columns: [f32; 2],
    rows: [[f32; 2]; 2],
    bottom: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            positions: [1, 0, 6],
            widths: [0.2; 2],
            columns: [0.5; 2],
            rows: [[0.5; 2]; 2],
            bottom: 0.3,
        }
    }
}

impl Layout {
    pub fn valid(&self) -> bool {
        (0.15..=0.85).contains(&self.bottom)
            && self.positions.iter().all(|&position| position < 8)
            && self.widths.iter().all(|width| (0.1..=0.35).contains(width))
            && self
                .columns
                .iter()
                .chain(self.rows.iter().flatten())
                .all(|ratio| (0.15..=0.85).contains(ratio))
    }
}

#[derive(Default)]
pub(super) struct State {
    pub layout: Layout,
    pub hidden: bool,
    applied: Option<[usize; 3]>,
    pending_tab: Option<&'static str>,
    bottom_selected: usize,
    bottom_collapsed: bool,
    pending_bottom: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
struct Region {
    position: [f32; 2],
    size: [f32; 2],
}

impl Region {
    fn split(self, axis: usize, ratio: f32, gap: f32) -> [Self; 2] {
        let start = self.position[axis].round();
        let end = (self.position[axis] + self.size[axis]).round();
        let length = (end - start - gap).max(2.0);
        let first = (length * ratio).clamp(1.0, length - 1.0);
        let boundary = (start + first).round();
        let mut a = self;
        let mut b = self;

        a.position[axis] = start;
        a.size[axis] = boundary - start;
        b.position[axis] = (boundary + gap).round();
        b.size[axis] = (end - b.position[axis]).max(1.0);

        [a, b]
    }
}

impl State {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            ..Self::default()
        }
    }

    pub fn toggle(&mut self) {
        self.hidden = !self.hidden;
    }

    pub fn position_menu(&mut self, ui: &dear_imgui_rs::Ui, panel: usize) {
        let titles = PANEL_NAMES
            .iter()
            .enumerate()
            .filter(|(index, _)| self.layout.positions[*index] == self.layout.positions[panel])
            .map(|(_, &name)| name)
            .collect::<Vec<_>>();

        let (activated, position) = kinematic_ui::docking::position_menu(
            ui,
            PANEL_NAMES[panel],
            &titles,
            self.layout.positions[panel],
        );

        if let Some(index) = activated {
            self.pending_tab = Some(titles[index]);
        }
        if let Some(position) = position {
            self.layout.positions[panel] = position;
        }
    }

    pub fn bottom_panel(&mut self, ui: &dear_imgui_rs::Ui, title: &str, content: impl FnOnce()) {
        if let Some(activated) = kinematic_ui::windows::panel_with_footer(
            ui,
            title,
            !self.bottom_collapsed,
            content,
            || {
                kinematic_ui::docking::footer(
                    ui,
                    (!self.bottom_collapsed).then_some(self.bottom_selected),
                    &BOTTOM_PANELS,
                    env!("CARGO_PKG_VERSION"),
                )
            },
        ) {
            if let Some(index) = activated {
                self.pending_bottom = Some(index);
            }
        }
    }

    fn select_bottom(&mut self, index: usize) {
        self.bottom_collapsed = !self.bottom_collapsed && self.bottom_selected == index;
        self.bottom_selected = index;
        if !self.bottom_collapsed {
            self.pending_tab = Some(BOTTOM_PANELS[index]);
        }
    }

    fn occupied(&self, side: usize, column: usize, row: usize) -> bool {
        self.layout
            .positions
            .contains(&(side * 4 + column * 2 + row))
    }

    pub fn draw(&mut self, ui: &dear_imgui_rs::Ui, gap: f32) {
        let initial_layout = self.applied.is_none();

        if let Some(index) = self.pending_bottom.take() {
            self.select_bottom(index);
        }
        if let Some(title) = self.pending_tab.take() {
            kinematic_ui::docking::activate(
                ui,
                title,
                &PANEL_NAMES
                    .into_iter()
                    .chain(BOTTOM_PANELS)
                    .collect::<Vec<_>>(),
            );
        }

        kinematic_ui::docking::background(ui);

        let viewport = ui.main_viewport();
        let size = viewport.work_size();
        let workspace = Region {
            position: viewport.work_pos(),
            size: size.map(|value| value.max(1.0)),
        };
        let mut center = workspace;
        let mut slots = [None; 8];
        let bottom_height = ui.frame_height() + ui.clone_style().window_padding()[1] * 2.0;
        let _padding = ui.push_style_var(StyleVar::WindowPadding([0.0; 2]));
        let _rounding = ui.push_style_var(StyleVar::WindowRounding(0.0));
        let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
        let background = ui.style_color(dear_imgui_rs::StyleColor::DockingEmptyBg);
        let _separator = ui.push_style_color(dear_imgui_rs::StyleColor::Separator, background);
        let _separator_hovered =
            ui.push_style_color(dear_imgui_rs::StyleColor::SeparatorHovered, background);
        let _separator_active =
            ui.push_style_color(dear_imgui_rs::StyleColor::SeparatorActive, background);

        host(ui, "Workspace host", workspace, true, || {
            if self.hidden {
                return;
            }

            for side in 0..2 {
                if !(0..2).any(|column| (0..2).any(|row| self.occupied(side, column, row))) {
                    continue;
                }

                let width = workspace.size[0] * self.layout.widths[side];
                let mut sidebar = workspace;
                sidebar.size[0] = width;

                if side == 0 {
                    center.position[0] += width + gap;
                } else {
                    sidebar.position[0] += workspace.size[0] - width;
                }

                center.size[0] -= width + gap;

                let position = [
                    if side == 0 {
                        sidebar.position[0] + width
                    } else {
                        sidebar.position[0] - gap
                    },
                    sidebar.position[1],
                ];
                let delta = kinematic_ui::docking::splitter(
                    ui,
                    &format!("Sidebar {side}"),
                    position,
                    [gap, sidebar.size[1]],
                    0,
                );
                self.layout.widths[side] = (self.layout.widths[side]
                    + if side == 0 { delta } else { -delta } / workspace.size[0])
                    .clamp(0.1, 0.35);

                let used = [0, 1].map(|column| (0..2).any(|row| self.occupied(side, column, row)));
                let columns = if used == [true, true] {
                    split(
                        ui,
                        &format!("Columns {side}"),
                        sidebar,
                        0,
                        &mut self.layout.columns[side],
                        gap,
                    )
                } else {
                    [sidebar; 2]
                };

                for column in 0..2 {
                    if !used[column] {
                        continue;
                    }

                    let rows = if self.occupied(side, column, 0) && self.occupied(side, column, 1) {
                        split(
                            ui,
                            &format!("Rows {side} {column}"),
                            columns[column],
                            1,
                            &mut self.layout.rows[side][column],
                            gap,
                        )
                    } else {
                        [columns[column]; 2]
                    };

                    for row in 0..2 {
                        if self.occupied(side, column, row) {
                            slots[side * 4 + column * 2 + row] = Some(rows[row]);
                        }
                    }
                }
            }
        });

        for (slot, region) in slots.into_iter().enumerate() {
            let changed = self.applied.is_none_or(|previous| {
                (0..PANEL_NAMES.len()).any(|panel| {
                    (previous[panel] == slot) != (self.layout.positions[panel] == slot)
                })
            });
            let windows = PANEL_NAMES
                .into_iter()
                .enumerate()
                .filter(|(panel, _)| self.layout.positions[*panel] == slot)
                .map(|(_, name)| dear_imgui_rs::WindowKey::new(name, name).unwrap());
            let layout = DockLayout::tabs(windows);

            dockspace(
                ui,
                &format!("Sidebar slot {slot}"),
                region.unwrap_or(workspace),
                &layout,
                changed,
                region.is_none(),
            );
        }

        let mut ratio = 1.0 - self.layout.bottom;
        let regions = if self.bottom_collapsed {
            split_at_bottom(center, bottom_height, gap)
        } else {
            split(ui, "Preview and bottom panels", center, 1, &mut ratio, gap)
        };
        self.layout.bottom = 1.0 - ratio;

        dockspace(
            ui,
            "Workspace center",
            regions[0],
            &tabs([preview::WINDOW_NAME]),
            self.applied.is_none(),
            false,
        );
        dockspace(
            ui,
            "Workspace bottom",
            regions[1],
            &tabs(BOTTOM_PANELS),
            self.applied.is_none(),
            false,
        );

        self.applied = Some(self.layout.positions);

        if initial_layout {
            self.pending_tab = Some(BOTTOM_PANELS[self.bottom_selected]);
        }
    }
}

fn split(
    ui: &dear_imgui_rs::Ui,
    id: &str,
    region: Region,
    axis: usize,
    ratio: &mut f32,
    gap: f32,
) -> [Region; 2] {
    let regions = region.split(axis, *ratio, gap);
    let mut position = region.position;
    let mut size = region.size;

    position[axis] += regions[0].size[axis];
    size[axis] = gap;

    let delta = kinematic_ui::docking::splitter(ui, id, position, size, axis);
    *ratio = (*ratio + delta / region.size[axis].max(1.0)).clamp(0.15, 0.85);

    regions
}

fn split_at_bottom(region: Region, height: f32, gap: f32) -> [Region; 2] {
    let bottom = (region.position[1] + region.size[1]).round();
    let top = (bottom - height).round();
    let mut upper = region;
    upper.size[1] = (top - region.position[1] - gap).max(1.0);

    let lower = Region {
        position: [region.position[0], top],
        size: [region.size[0], bottom - top],
    };

    [upper, lower]
}

fn host(ui: &dear_imgui_rs::Ui, name: &str, region: Region, hidden: bool, content: impl FnOnce()) {
    ui.window(name)
        .position(region.position, Condition::Always)
        .size(region.size, Condition::Always)
        .flags(
            (if hidden {
                WindowFlags::NO_INPUTS
            } else {
                WindowFlags::empty()
            }) | WindowFlags::NO_BACKGROUND
                | WindowFlags::NO_DECORATION
                | WindowFlags::NO_MOVE
                | WindowFlags::NO_DOCKING
                | WindowFlags::NO_SAVED_SETTINGS
                | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                | WindowFlags::NO_NAV_FOCUS,
        )
        .build(content);
}

fn dockspace(
    ui: &dear_imgui_rs::Ui,
    name: &str,
    region: Region,
    layout: &DockLayout,
    changed: bool,
    hidden: bool,
) {
    host(ui, name, region, hidden, || {
        let flags = DockNodeFlags::NO_UNDOCKING | DockNodeFlags::NO_DOCKING_SPLIT;
        let flags = if hidden {
            flags | DockNodeFlags::KEEP_ALIVE_ONLY
        } else {
            flags
        };

        ui.dockspace()
            .root_id(ui.get_id("Dockspace"))
            .current_window(region.size)
            .flags(flags)
            .layout(
                layout,
                if changed {
                    DockLayoutApply::Replace
                } else {
                    DockLayoutApply::IfMissing
                },
            )
            .build()
            .expect("Workspace layout must be valid.");
    });
}

fn tabs<const N: usize>(names: [&str; N]) -> DockLayout {
    DockLayout::tabs(names.map(|name| dear_imgui_rs::WindowKey::new(name, name).unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_layout_validates_saved_positions_and_sizes() {
        let mut layout = Layout::default();

        assert!(layout.valid());

        layout.positions[0] = 8;

        assert!(!layout.valid());

        layout.positions[0] = 0;
        layout.rows[0][0] = f32::NAN;

        assert!(!layout.valid());
    }

    #[test]
    fn workspace_background_does_not_cover_dock_contents() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context
            .io_mut()
            .set_config_flags(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        let mut state = State::default();

        let mut file_click_position = None;
        let mut clicked = false;

        for frame in 0..6 {
            if let Some(position) = file_click_position {
                context.io_mut().add_mouse_pos_event(position);
                context
                    .io_mut()
                    .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, frame == 3);
            }

            let ui = context.frame();

            state.draw(ui, GAP);

            if frame == 0 {
                assert_eq!(state.pending_tab, Some(timeline::WINDOW_NAME));
            }

            let mut samples = Vec::new();

            for name in PANEL_NAMES.into_iter().chain([
                preview::WINDOW_NAME,
                timeline::WINDOW_NAME,
                code::CONSOLE_WINDOW,
            ]) {
                kinematic_ui::windows::panel(ui, name, || {
                    assert!(ui.is_window_docked(), "{name} must be docked.");

                    let origin = ui.cursor_screen_pos();
                    let min = [origin[0] + 10.0, origin[1] + 10.0];
                    let max = [min[0] + 20.0, min[1] + 20.0];

                    ui.get_window_draw_list()
                        .add_rect(min, max, [1.0, 0.0, 1.0, 1.0])
                        .filled(true)
                        .build();
                    let point = [min[0] + 10.0, min[1] + 10.0];

                    if name == code::FILES_WINDOW {
                        file_click_position = Some(point);
                        ui.set_cursor_screen_pos(min);
                        clicked |= ui.invisible_button("Panel action", [20.0; 2]);
                    }

                    samples.push((name, point));
                });
            }

            ui.with_bound_context(|| unsafe {
                let host = dear_imgui_rs::sys::igFindWindowByName(c"Workspace host".as_ptr());

                dear_imgui_rs::sys::igBringWindowToDisplayFront(host);
            });

            let rendered = context.render_legacy();

            if frame < 3 {
                continue;
            }

            assert_eq!(samples.len(), 5);

            for (name, point) in samples {
                let mut color = None;

                for list in rendered.draw_data().draw_lists() {
                    for command in list.commands() {
                        let dear_imgui_rs::render::DrawCmd::Elements { count, cmd_params } =
                            command
                        else {
                            continue;
                        };
                        let clip = cmd_params.clip_rect;

                        if point[0] < clip[0]
                            || point[0] >= clip[2]
                            || point[1] < clip[1]
                            || point[1] >= clip[3]
                        {
                            continue;
                        }

                        for triangle in list.idx_buffer()
                            [cmd_params.idx_offset..cmd_params.idx_offset + count]
                            .chunks_exact(3)
                        {
                            let vertices = [0, 1, 2].map(|i| {
                                list.vtx_buffer()[cmd_params.vtx_offset + triangle[i] as usize]
                            });
                            let signs = [0, 1, 2].map(|i| {
                                let a = vertices[i].pos;
                                let b = vertices[(i + 1) % 3].pos;
                                (b[0] - a[0]) * (point[1] - a[1])
                                    - (b[1] - a[1]) * (point[0] - a[0])
                            });

                            if signs.iter().sum::<f32>().abs() <= f32::EPSILON {
                                continue;
                            }

                            if signs.iter().all(|&value| value >= 0.0)
                                || signs.iter().all(|&value| value <= 0.0)
                            {
                                if vertices.iter().all(|vertex| vertex.col == vertices[0].col)
                                    && vertices[0].col >> 24 == 255
                                {
                                    color = Some(vertices[0].col);
                                }
                            }
                        }
                    }
                }

                assert_eq!(
                    color,
                    Some(0xffff00ff),
                    "{name} content must remain visible."
                );
            }
        }
        assert!(clicked, "Workspace hosts must not intercept panel input.");
    }

    #[test]
    fn title_rows_switch_panels_without_native_tab_bars() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context
            .io_mut()
            .set_config_flags(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        let mut state = State::default();
        state.layout.positions = [0; 3];
        let mut click = None;

        let mut expanded_height = 0.0;
        let mut collapsed_height = 0.0;

        for (step, target) in PANEL_NAMES
            .into_iter()
            .chain([
                code::CONSOLE_WINDOW,
                code::CONSOLE_WINDOW,
                timeline::WINDOW_NAME,
            ])
            .enumerate()
        {
            for frame in 0..7 {
                if let Some(position) = click {
                    context.io_mut().add_mouse_pos_event(position);
                    context
                        .io_mut()
                        .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, frame == 2);
                }

                let ui = context.frame();
                let _style = kinematic_ui::windows::style(ui, 8.0, GAP);

                state.draw(ui, GAP);

                let mut visible = Vec::new();

                for (names, sidebar) in [(&PANEL_NAMES[..], true), (&BOTTOM_PANELS[..], false)] {
                    for (panel, &name) in names.iter().enumerate() {
                        if !sidebar {
                            state.bottom_panel(ui, name, || visible.push(name));
                            continue;
                        }
                        kinematic_ui::windows::panel(ui, name, || {
                            let origin = ui.cursor_screen_pos();

                            if frame >= 3 {
                                assert!(
                                    (origin[1]
                                        - ui.window_pos()[1]
                                        - kinematic_ui::theme::WINDOW_PADDING[1])
                                        .abs()
                                        < 1.0,
                                    "The native tab bar must not reserve a second title row."
                                );
                            }
                            if sidebar && names.contains(&target) {
                                let target_index =
                                    names.iter().position(|&name| name == target).unwrap();
                                let spacing = ui.clone_style().item_spacing()[0];
                                let full_width = names
                                    .iter()
                                    .map(|name| ui.calc_text_size(name)[0])
                                    .sum::<f32>();
                                let available = ui.content_region_avail_width()
                                    - if sidebar {
                                        ui.frame_height() + spacing
                                    } else {
                                        0.0
                                    };
                                let scale = ((available
                                    - spacing * names.len().saturating_sub(1) as f32)
                                    / full_width)
                                    .clamp(0.0, 1.0);
                                let offset = names[..target_index]
                                    .iter()
                                    .map(|name| ui.calc_text_size(name)[0] * scale + spacing)
                                    .sum::<f32>();

                                click = Some([
                                    origin[0] + offset + ui.calc_text_size(target)[0] * scale * 0.5,
                                    origin[1] + ui.frame_height() * 0.5,
                                ]);
                            }

                            if sidebar {
                                state.position_menu(ui, panel);
                            }
                            visible.push(name);
                        });
                    }
                }
                kinematic_ui::windows::viewport_panel(ui, preview::WINDOW_NAME, || {
                    if frame == 6 {
                        if step == 3 {
                            expanded_height = ui.window_size()[1];
                        }
                        if step == 4 {
                            collapsed_height = ui.window_size()[1];
                        }
                        if step == 5 {
                            assert_eq!(ui.window_size()[1], expanded_height);
                        }
                    }
                });

                if BOTTOM_PANELS.contains(&target) {
                    ui.window(BOTTOM_PANELS[state.bottom_selected]).build(|| {
                        let position = ui.window_pos();

                        assert!(ui.is_window_docked());
                        assert!(position[0] > 0.0);
                        assert!(ui.window_size()[0] < 1280.0);
                        if state.bottom_collapsed {
                            let required =
                                ui.frame_height() + ui.clone_style().window_padding()[1] * 2.0;
                            assert!(
                                ui.window_size()[1] >= required - 1.0,
                                "The collapsed dock must fit its footer and window padding."
                            );
                        }
                        assert_eq!(position[1] + ui.window_size()[1], 720.0);
                        let index = BOTTOM_PANELS
                            .iter()
                            .position(|&name| name == target)
                            .unwrap();
                        let offset = BOTTOM_PANELS[..index]
                            .iter()
                            .map(|name| {
                                ui.calc_text_size(name)[0] + ui.clone_style().item_spacing()[0]
                            })
                            .sum::<f32>();
                        click = Some([
                            position[0]
                                + kinematic_ui::theme::WINDOW_PADDING[0]
                                + offset
                                + ui.calc_text_size(target)[0] * 0.5,
                            position[1] + ui.window_size()[1]
                                - kinematic_ui::theme::WINDOW_PADDING[1]
                                - ui.frame_height() * 0.5,
                        ]);
                    });
                }

                if frame == 6 {
                    if step == 4 {
                        assert!(state.bottom_collapsed);
                        assert_eq!(visible.len(), 1);
                        assert!(collapsed_height > expanded_height);
                    } else {
                        assert!(
                            visible.contains(&target),
                            "Clicking {target} must select its panel."
                        );
                        assert_eq!(visible.len(), 2);
                    }
                }

                drop(_style);
                context.end_frame();
            }
        }
    }

    #[test]
    fn hiding_sidebars_expands_the_center_and_restores_panel_positions() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context
            .io_mut()
            .set_config_flags(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_display_size([1280.0, 720.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        let mut state = State::default();
        let mut widths = Vec::new();
        let mut panel_regions = Vec::new();

        for hidden in [false, true, false] {
            state.hidden = hidden;

            for _ in 0..3 {
                let ui = context.frame();

                state.draw(ui, GAP);

                if !hidden {
                    let mut regions = Vec::new();

                    for name in PANEL_NAMES {
                        kinematic_ui::windows::panel(ui, name, || {
                            assert!(ui.is_window_docked(), "{name} must be docked.");
                            ui.text(name);
                            regions.push((ui.window_pos(), ui.window_size()))
                        });
                    }

                    panel_regions.push(regions);
                }

                for name in [
                    preview::WINDOW_NAME,
                    timeline::WINDOW_NAME,
                    code::CONSOLE_WINDOW,
                ] {
                    kinematic_ui::windows::panel(ui, name, || {});
                }

                let mut width = 0.0;

                ui.window("Workspace center")
                    .build(|| width = ui.window_size()[0]);
                widths.push(width);
                context.end_frame();
            }
        }

        assert!(widths[5] > widths[2]);
        assert_eq!(widths[5], 1280.0);
        assert_eq!(widths[8], widths[2]);
        assert_eq!(panel_regions[2].len(), PANEL_NAMES.len());
        assert_eq!(panel_regions[5].len(), PANEL_NAMES.len());
        assert_eq!(panel_regions[5], panel_regions[2]);

        for slot in 0..8 {
            state.layout.positions[0] = slot;

            for _ in 0..3 {
                let ui = context.frame();

                state.draw(ui, GAP);

                for name in PANEL_NAMES {
                    kinematic_ui::windows::panel(ui, name, || {});
                }

                for name in [
                    preview::WINDOW_NAME,
                    timeline::WINDOW_NAME,
                    code::CONSOLE_WINDOW,
                ] {
                    kinematic_ui::windows::panel(ui, name, || {});
                }

                context.end_frame();
            }
        }

        let width = state.layout.widths[0];
        let boundary = 1280.0 * width + GAP * 0.5;

        for (x, down) in [
            (boundary, false),
            (boundary, true),
            (boundary + 32.0, true),
            (boundary + 32.0, false),
        ] {
            context.io_mut().add_mouse_pos_event([x, 300.0]);
            context
                .io_mut()
                .add_mouse_button_event(dear_imgui_rs::MouseButton::Left, down);

            let ui = context.frame();

            state.draw(ui, GAP);

            for name in PANEL_NAMES {
                kinematic_ui::windows::panel(ui, name, || {});
            }

            for name in [
                preview::WINDOW_NAME,
                timeline::WINDOW_NAME,
                code::CONSOLE_WINDOW,
            ] {
                kinematic_ui::windows::panel(ui, name, || {});
            }

            context.end_frame();
        }

        assert!(state.layout.widths[0] > width);
    }
}
