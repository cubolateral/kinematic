//! Centered messages with a single acknowledgement button.

use std::collections::VecDeque;

use dear_imgui_rs::Ui;

use crate::{icons, widgets, windows};

pub enum Kind {
    Information,
    Warning,
    Error,
}

#[derive(Default)]
pub struct State {
    pending: VecDeque<(Kind, String)>,
}

impl State {
    pub fn show(&mut self, kind: Kind, message: impl Into<String>) {
        self.pending.push_back((kind, message.into()));
    }

    pub fn is_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn draw(&mut self, ui: &Ui) {
        let Some((kind, message)) = self.pending.front() else {
            return;
        };
        let (icon, title) = match kind {
            Kind::Information => (icons::INFO, "Information"),
            Kind::Warning => (icons::WARNING, "Warning"),
            Kind::Error => (icons::CIRCLE_X, "Error"),
        };
        let title = format!("{icon}  {title}###MessageBox");

        if !ui.is_popup_open(&title) {
            ui.open_popup(&title);
        }

        if let Some(_popup) = windows::modal_popup(ui, &title, None, [480.0, 0.0]) {
            widgets::message(ui, message);

            if windows::actions(ui, &[("OK", true)]).is_some() {
                ui.close_current_popup();

                self.pending.pop_front();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_stay_modal_until_acknowledged() {
        let _lock = crate::TEST_LOCK.lock().unwrap();
        let mut context = dear_imgui_rs::Context::create();

        context.set_ini_filename(None::<&str>).unwrap();
        context.io_mut().set_display_size([800.0, 600.0]);
        context.io_mut().set_delta_time(1.0 / 60.0);
        context
            .font_atlas()
            .try_claim_legacy_renderer()
            .unwrap()
            .build();

        let mut state = State::default();

        state.show(Kind::Error, "Project is unavailable.");

        for frame in 0..3 {
            context
                .io_mut()
                .add_key_event(dear_imgui_rs::Key::Escape, frame == 1);

            let ui = context.frame();

            state.draw(ui);

            assert!(state.is_pending());
            assert!(ui.is_popup_open(&format!("{}  Error###MessageBox", icons::CIRCLE_X)));

            context.end_frame();
        }
    }
}
