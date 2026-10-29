//! Code presentation and the current CTE rendering integration.

pub mod documentation;
pub mod popup;
pub mod text;

#[derive(Clone, Copy, Default)]
pub struct TextActions {
    pub select_all: bool,
    pub copy: bool,
}

pub fn palette() -> dear_imgui_cte::Palette {
    use dear_imgui_cte::{Palette, PaletteColor};
    let mut palette = Palette::dark();
    for (slot, color) in [
        (PaletteColor::Text, 0xffd4d4d4),
        (PaletteColor::Identifier, 0xffd4d4d4),
        (PaletteColor::Keyword, 0xffd69c56),
        (PaletteColor::Declaration, 0xffb0c94e),
        (PaletteColor::Number, 0xffa8ceb5),
        (PaletteColor::String, 0xff7891ce),
        (PaletteColor::Comment, 0xff55996a),
        (PaletteColor::Background, 0),
        (PaletteColor::LineNumber, 0xff858585),
        (PaletteColor::Selection, text::SELECTION_COLOR),
    ] {
        palette.set(slot, color);
    }
    palette
}

pub fn marker(
    editor: &mut dear_imgui_cte::TextEditor,
    line: usize,
    error: bool,
) -> dear_imgui_cte::CteResult<()> {
    let color = if error { 0xff5555f1 } else { 0xff55cde2 };
    let background = if error { 0x445555f1 } else { 0x4455cde2 };
    editor.add_marker(line, color, background, "", "")
}
