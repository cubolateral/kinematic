use super::documentation::Documentation;
use dear_imgui_cte::{Palette, PaletteColor, Position, TextEditor};
use dear_imgui_rs::{Condition, StyleColor, StyleVar, Ui, WindowFlags};

pub(super) fn window<R>(
    ui: &Ui,
    name: &str,
    position: [f32; 2],
    size: Option<[f32; 2]>,
    flags: WindowFlags,
    content: impl FnOnce() -> R,
) -> Option<R> {
    let _rounding = ui.push_style_var(StyleVar::WindowRounding(7.0));
    let _padding = ui.push_style_var(StyleVar::WindowPadding([12.0, 12.0]));
    let _border = ui.push_style_var(StyleVar::WindowBorderSize(1.0));
    let background = ui.clone_style().color(StyleColor::WindowBg);
    let _background = ui.push_style_color(StyleColor::WindowBg, background);
    let _child = ui.push_style_color(StyleColor::ChildBg, background);
    let mut window = ui.window(name).position(position, Condition::Always).flags(
        WindowFlags::NO_DECORATION
            | WindowFlags::NO_MOVE
            | WindowFlags::NO_FOCUS_ON_APPEARING
            | WindowFlags::NO_DOCKING
            | WindowFlags::NO_SAVED_SETTINGS
            | flags,
    );
    if let Some(size) = size {
        window = window.size(size, Condition::Always);
    }
    window.build(content)
}

pub(super) fn editor(
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

pub(super) struct Content {
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

    pub fn draw(&mut self, ui: &Ui, font_size: f32) -> Result<(), String> {
        self.view.draw(ui, &mut self.buffer, font_size)
    }

    pub fn height(&self) -> f32 {
        self.view.height
    }

    pub fn packed_color(&self, slot: PaletteColor) -> u32 {
        self.buffer.palette().get(slot)
    }
}
