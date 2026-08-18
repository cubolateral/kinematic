use crate::{
    core::components::{Inspection, Node},
    editor::Editor,
};

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    ui.window("Inspector").build(|| {
        let world = editor.get_scene().get_world();

        for (entity, node, inspection) in world.query::<(hecs::Entity, &Node, &Inspection)>().iter()
        {
            if !node.is_activated {
                continue;
            }

            if !ui.collapsing_header(
                format!("{}##entity_{}", inspection.object_name, entity.id()),
                dear_imgui_rs::TreeNodeFlags::NONE,
            ) {
                continue;
            }

            for trackable in (inspection.get)(&world, entity) {
                ui.separator_with_text(trackable.name);

                for track in (trackable.get)() {
                    property(ui, track.name, &(track.get)(&world, entity).to_string());
                }

                ui.spacing();
            }

            ui.spacing();
            ui.separator();
            ui.spacing();
        }
    });
}

fn property(ui: &dear_imgui_rs::Ui, name: &str, value: &str) {
    ui.text(name);
    ui.same_line();

    let width = ui
        .current_font()
        .calc_text_size(ui.current_font_size(), f32::MAX, f32::MAX, value)[0];
    ui.set_cursor_pos_x(ui.cursor_pos_x() + ui.content_region_avail()[0] - width);
    ui.text_disabled(value);
}
