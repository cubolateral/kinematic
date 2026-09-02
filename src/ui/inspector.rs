use crate::{
    core::components::{Inspection, Name, Node},
    editor::Editor,
};

use super::widgets::text_size;

pub(super) const WINDOW_NAME: &str = "Inspector";

pub(super) fn draw(editor: &mut Editor, ui: &dear_imgui_rs::Ui) {
    let selected = editor.get_selected_entity();

    ui.window(WINDOW_NAME).build(|| {
        let Some(entity) = selected else {
            ui.text_wrapped("Select an object from the Scene Tree or Timeline.");
            return;
        };

        let world = editor.get_scene().get_world();
        let Ok(inspection) = world.get::<&Inspection>(entity) else {
            ui.text_disabled("The selected object is unavailable.");
            return;
        };
        let name = world
            .get::<&Name>(entity)
            .expect("Inspected object must contain a Name component.");
        let node = world
            .get::<&Node>(entity)
            .expect("Inspected object must contain a Node component.");

        ui.text(name.get());
        ui.same_line();
        ui.text_disabled(if node.is_activated {
            "Active."
        } else {
            "Inactive."
        });
        ui.text_disabled(format!("Entity ID: {}.", entity.to_bits()));
        ui.text_disabled(format!("Object type: {}.", inspection.object_name));
        ui.separator();

        for trackable in (inspection.get)(&world, entity) {
            ui.separator_with_text(trackable.name);

            for track in (trackable.get)() {
                property(ui, track.name, &(track.get)(&world, entity).to_string());
            }

            ui.spacing();
        }
    });
}

fn property(ui: &dear_imgui_rs::Ui, name: &str, value: &str) {
    ui.text(name);
    ui.same_line();

    let width = text_size(ui, value)[0];
    ui.set_cursor_pos_x(ui.cursor_pos_x() + ui.content_region_avail()[0] - width);
    ui.text_disabled(value);
}
