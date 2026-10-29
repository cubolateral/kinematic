use super::documentation::Documentation;
use dear_imgui_cte::{Palette, PaletteColor, Position, TextEditor};
use dear_imgui_rs::Ui;

pub use crate::windows::anchored as window;

pub fn editor(
    context: &dear_imgui_rs::Context,
    palette: &Palette,
) -> dear_imgui_cte::CteResult<TextEditor> {
    let mut editor = TextEditor::try_create(context)?;

    editor.set_language(None);
    editor.set_palette(palette)?;
    editor.set_read_only(true);
    editor.set_show_line_numbers(false);
    editor.set_show_minimap(false);
    editor.set_show_whitespaces(false);
    editor.set_show_spaces(false);
    editor.set_show_tabs(false);
    editor.set_word_wrap_enabled(true);

    Ok(editor)
}

pub struct Content {
    buffer: TextEditor,
    view: Documentation,
    source: String,
}

impl Content {
    pub fn new(
        context: &dear_imgui_rs::Context,
        palette: &Palette,
    ) -> dear_imgui_cte::CteResult<Self> {
        Ok(Self {
            buffer: editor(context, palette)?,
            view: Documentation::default(),
            source: String::new(),
        })
    }

    pub fn set(&mut self, markdown: &str) -> Result<(), String> {
        if self.source != markdown {
            self.view = Documentation::parse(markdown);

            self.buffer
                .set_text(&self.view.text)
                .and_then(|_| self.buffer.set_cursor(Position::new(0, 0)))
                .map_err(|error| error.to_string())?;

            self.source = markdown.to_owned();
        }

        Ok(())
    }

    pub fn draw(
        &mut self,
        ui: &Ui,
        font_size: f32,
        actions: super::TextActions,
    ) -> Result<(), String> {
        self.view.draw(ui, &mut self.buffer, font_size, actions)
    }

    pub fn height(&self) -> f32 {
        self.view.height
    }

    pub fn packed_color(&self, slot: PaletteColor) -> u32 {
        self.buffer.palette().get(slot)
    }
}
