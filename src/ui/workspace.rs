use super::{code, inspector, preview, scene_tree, timeline};
use dear_imgui_rs::{Condition, DockLayout, DockLayoutApply, DockSplit, StyleVar, WindowFlags};

pub(super) fn draw(ui: &dear_imgui_rs::Ui) {
    let layout = DockLayout::split(
        DockSplit::Down,
        0.35,
        DockLayout::split(
            DockSplit::Left,
            0.25,
            tabs([code::FILES_WINDOW]),
            tabs([timeline::WINDOW_NAME, code::CONSOLE_WINDOW]),
        ),
        DockLayout::split(
            DockSplit::Right,
            0.75,
            DockLayout::split(
                DockSplit::Left,
                0.5,
                tabs([code::WINDOW_NAME]),
                tabs([preview::WINDOW_NAME]),
            ),
            tabs([scene_tree::WINDOW_NAME, inspector::WINDOW_NAME]),
        ),
    );

    let viewport = ui.main_viewport();
    let size = viewport.work_size();
    let size = [
        size[0].max(1.0),
        (size[1] - kinematic_ui::status_bar::height(ui)).max(1.0),
    ];
    let root = ui.get_id("Workspace");
    let _padding = ui.push_style_var(StyleVar::WindowPadding([0.0; 2]));
    let _rounding = ui.push_style_var(StyleVar::WindowRounding(0.0));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
    ui.window("Workspace host")
        .position(viewport.work_pos(), Condition::Always)
        .size(size, Condition::Always)
        .flags(
            WindowFlags::NO_DECORATION
                | WindowFlags::NO_MOVE
                | WindowFlags::NO_DOCKING
                | WindowFlags::NO_SAVED_SETTINGS
                | WindowFlags::NO_BRING_TO_FRONT_ON_FOCUS
                | WindowFlags::NO_NAV_FOCUS,
        )
        .build(|| {
            ui.dockspace()
                .root_id(root)
                .current_window(size)
                .layout(&layout, DockLayoutApply::IfMissing)
                .build()
                .expect("Workspace layout must be valid.");
        });
}

fn tabs<const N: usize>(names: [&str; N]) -> DockLayout {
    DockLayout::tabs(names.map(|name| dear_imgui_rs::WindowKey::new(name, name).unwrap()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn resizing_keeps_the_workspace_above_the_status_bar() {
        let _lock = crate::ui::IMGUI_TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();
        context.set_ini_filename(None::<&str>).unwrap();
        context
            .io_mut()
            .set_config_flags(dear_imgui_rs::ConfigFlags::DOCKING_ENABLE);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        for size in [[1280.0, 720.0], [640.0, 360.0]] {
            context.io_mut().set_display_size(size);
            for _ in 0..3 {
                let ui = context.frame();
                let _menu = ui.begin_main_menu_bar();
                drop(_menu);
                super::draw(ui);
                kinematic_ui::status_bar::draw(ui, "Project saved.", env!("CARGO_PKG_VERSION"));

                let mut workspace_bottom = 0.0;
                ui.window("Workspace host").build(|| {
                    workspace_bottom = ui.window_pos()[1] + ui.window_size()[1];
                });
                ui.window("Project status").build(|| {
                    assert_eq!(ui.window_pos()[1], workspace_bottom);
                    assert_eq!(ui.window_pos()[1] + ui.window_size()[1], size[1]);
                    assert_eq!(ui.window_size()[0], size[0]);
                });
                context.end_frame();
            }
        }
    }
}
