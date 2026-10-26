use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ratex_font::{FontId, katex_ttf_glyph_char};
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_types::{display_item::DisplayItem, path_command::PathCommand};

use crate::core::types::{Color, Vector2};

pub(super) struct FormulaPart {
    pub path: skia_safe::Path,
    pub color: Option<Color>,
    pub key: String,
    pub group: usize,
}

pub(super) struct FormulaGeometry {
    pub size: Vector2,
    pub parts: Vec<FormulaPart>,
}

#[derive(Default)]
struct FormulaCache {
    fonts: HashMap<FontId, skia_safe::Font>,
    formulas: HashMap<String, Rc<FormulaGeometry>>,
}

impl FormulaCache {
    fn glyph(&mut self, font: &str, code: u32) -> Option<skia_safe::Path> {
        let id = FontId::parse(font).expect("Math font is unknown.");
        let font = self.fonts.entry(id).or_insert_with(|| {
            let bytes = ratex_katex_fonts::ttf_bytes(&format!("KaTeX_{id}.ttf"))
                .expect("Math font is not embedded.");
            let typeface = skia_safe::FontMgr::new()
                .new_from_data(&bytes, None)
                .expect("Math font outlines could not be loaded.");

            skia_safe::Font::new(typeface, 1000.0)
        });
        let ch = katex_ttf_glyph_char(id, code);
        let glyph = font.unichar_to_glyph(ch as i32);

        assert!(glyph != 0, "Math glyph {ch} is missing.");

        font.get_path(glyph)
    }
}

thread_local! {
    static CACHE: RefCell<FormulaCache> = RefCell::new(FormulaCache::default());
}

// Cache in em units so size, color, and transform animation reuse the same layout.
pub(super) fn geometry(text: &str) -> Rc<FormulaGeometry> {
    CACHE.with_borrow_mut(|cache| {
        if let Some(geometry) = cache.formulas.get(text) {
            return geometry.clone();
        }

        let ast = ratex_parser::parser::parse(text)
            .unwrap_or_else(|error| panic!("Invalid LaTeX formula `{text}`: {error}."));
        let options = LayoutOptions::default();
        let mut boxes = layout(&ast, &options);
        let display = to_display_list(&boxes);
        let mut groups = Vec::new();
        tag_groups(&mut boxes, "", &mut groups);
        let tagged = to_display_list(&boxes);
        // Resolve inherited fill separately from explicit colors, including black and white.
        // Both layout passes run only when a formula first enters the cache.
        let alternate = to_display_list(&layout(
            &ast,
            &options.with_color(ratex_types::color::Color::WHITE),
        ));

        assert_eq!(display.items.len(), alternate.items.len());

        let mut geometry = FormulaGeometry {
            size: Vector2::new(display.width as f32, display.total_height() as f32),
            parts: Vec::new(),
        };

        let mut tokens = super::text_morph::TokenGroups::default();
        let mut previous_scope = "";
        let mut previous_run = None;
        assert_eq!(display.items.len(), tagged.items.len());
        for ((item, alternate), tagged) in display
            .items
            .iter()
            .zip(&alternate.items)
            .zip(&tagged.items)
        {
            let tint = item_color(item);
            let fill =
                (tint == item_color(alternate)).then(|| Color::new(tint.r, tint.g, tint.b, tint.a));

            let start = geometry.parts.len();
            emit(item, cache, fill, &mut geometry.parts);
            let (group, run) = &groups[item_color(tagged).r as usize];
            if group != previous_scope || *run != previous_run {
                tokens.next(' ');
                previous_scope = group;
                previous_run = *run;
            }
            let symbol = match item {
                DisplayItem::GlyphPath { char_code, .. } => {
                    char::from_u32(*char_code).unwrap_or(' ')
                }
                _ => ' ',
            };
            let token = tokens.next(symbol);
            let identity = match item {
                DisplayItem::GlyphPath {
                    font, char_code, ..
                } => format!("{font}:{char_code}"),
                DisplayItem::Line { .. } => "Line".to_owned(),
                DisplayItem::Rect { .. } => "Rect".to_owned(),
                DisplayItem::Path { .. } => "Path".to_owned(),
            };
            for (index, part) in geometry.parts[start..].iter_mut().enumerate() {
                part.key = format!("{group}/{identity}/{index}");
                part.group = token;
            }
        }

        let offset = -geometry.size * 0.5;

        for part in &mut geometry.parts {
            part.path = part.path.with_offset((offset.x, offset.y));

            // Include glyph overhangs in selection bounds.
            let bounds = part.path.compute_tight_bounds();

            geometry.size.x = geometry
                .size
                .x
                .max(bounds.left.abs().max(bounds.right.abs()) * 2.0);

            geometry.size.y = geometry
                .size
                .y
                .max(bounds.top.abs().max(bounds.bottom.abs()) * 2.0);
        }

        let geometry = Rc::new(geometry);

        if cache.formulas.len() >= 128 {
            cache.formulas.clear();
        }

        cache.formulas.insert(text.to_owned(), geometry.clone());

        geometry
    })
}

// A separate display pass carries group labels through RaTeX's own placement logic.
fn tag_groups(
    node: &mut ratex_layout::LayoutBox,
    group: &str,
    groups: &mut Vec<(String, Option<usize>)>,
) {
    use ratex_layout::layout_box::{BoxContent, VBoxChildKind};
    let color = ratex_types::color::Color::new(groups.len() as f32, 0.0, 0.0, 1.0);
    let run = matches!(node.content, BoxContent::GlyphRun { .. }).then_some(groups.len());
    groups.push((group.to_owned(), run));
    node.color = color;
    let mut child = |node: &mut ratex_layout::LayoutBox, role: &str| {
        tag_groups(node, &format!("{group}/{role}"), groups);
    };
    match &mut node.content {
        BoxContent::HBox(children) => {
            let mut counts = HashMap::new();
            for node in children {
                match &node.content {
                    BoxContent::Glyph { .. }
                    | BoxContent::GlyphRun { .. }
                    | BoxContent::SupSub { .. }
                    | BoxContent::OpLimits { .. }
                    | BoxContent::Kern
                    | BoxContent::Empty => {
                        tag_groups(node, group, groups);
                    }
                    BoxContent::HBox(children)
                        if children
                            .iter()
                            .filter(|child| {
                                !matches!(child.content, BoxContent::Kern | BoxContent::Empty)
                            })
                            .count()
                            <= 1 =>
                    {
                        tag_groups(node, group, groups);
                    }
                    _ => {
                        let kind = std::mem::discriminant(&node.content);
                        let index = counts.entry(kind).or_insert(0);
                        tag_groups(node, &format!("{group}/{kind:?}:{index}"), groups);
                        *index += 1;
                    }
                }
            }
        }
        BoxContent::VBox(children) => {
            for (index, node) in children.iter_mut().enumerate() {
                if let VBoxChildKind::Box(node) = &mut node.kind {
                    child(node, &format!("Row{index}"));
                }
            }
        }
        BoxContent::Fraction { numer, denom, .. } => {
            child(numer, "Numerator");
            child(denom, "Denominator");
        }
        BoxContent::SupSub { base, sup, sub, .. } | BoxContent::OpLimits { base, sup, sub, .. } => {
            let identity: Vec<_> = to_display_list(base)
                .items
                .iter()
                .filter_map(|item| match item {
                    DisplayItem::GlyphPath {
                        font, char_code, ..
                    } => Some(format!("{font}:{char_code}")),
                    _ => None,
                })
                .collect();
            if let Some(node) = sup {
                child(node, &format!("{identity:?}/Superscript"));
            }
            if let Some(node) = sub {
                child(node, &format!("{identity:?}/Subscript"));
            }
            tag_groups(base, group, groups);
        }
        BoxContent::Radical { body, index, .. } => {
            child(body, "Radicand");
            if let Some(node) = index {
                child(node, "RootIndex");
            }
        }
        BoxContent::Accent { base, accent, .. } => {
            child(base, "Base");
            child(accent, "Accent");
        }
        BoxContent::LeftRight { left, inner, right } => {
            child(left, "Left");
            child(inner, "Inner");
            child(right, "Right");
        }
        BoxContent::Array {
            cells, row_tags, ..
        } => {
            for (row, cells) in cells.iter_mut().enumerate() {
                for (column, node) in cells.iter_mut().enumerate() {
                    child(node, &format!("Cell{row}:{column}"));
                }
            }
            for (row, node) in row_tags.iter_mut().enumerate() {
                if let Some(node) = node {
                    child(node, &format!("Tag{row}"));
                }
            }
        }
        BoxContent::Framed {
            body,
            bg_color,
            border_color,
            ..
        } => {
            if bg_color.is_some() {
                *bg_color = Some(color);
            }
            *border_color = color;
            child(body, "Body");
        }
        BoxContent::RaiseBox { body, .. }
        | BoxContent::Scaled { body, .. }
        | BoxContent::Angl { body, .. }
        | BoxContent::Overline { body, .. }
        | BoxContent::Underline { body, .. } => child(body, "Body"),
        BoxContent::ProofTree { children, .. } => {
            for (index, node) in children.iter_mut().enumerate() {
                child(&mut node.box_, &format!("Proof{index}"));
            }
        }
        BoxContent::Glyph { .. }
        | BoxContent::GlyphRun { .. }
        | BoxContent::Rule { .. }
        | BoxContent::Kern
        | BoxContent::SvgPath { .. }
        | BoxContent::Empty => {}
    }
}

pub(super) fn morph_geometry(text: &str, thickness: f32) -> Vec<super::text_morph::MorphPart> {
    geometry(text)
        .parts
        .iter()
        .map(|part| super::text_morph::MorphPart {
            key: part.key.clone(),
            group: part.group,
            path: super::text_2d::weighted_path(&part.path, thickness),
            color: part.color,
        })
        .collect()
}

fn item_color(item: &DisplayItem) -> ratex_types::color::Color {
    match item {
        DisplayItem::GlyphPath { color, .. }
        | DisplayItem::Line { color, .. }
        | DisplayItem::Rect { color, .. }
        | DisplayItem::Path { color, .. } => *color,
    }
}

fn stroked(path: &skia_safe::Path, width: f32) -> skia_safe::Path {
    let mut paint = skia_safe::Paint::default();

    paint
        .set_style(skia_safe::PaintStyle::Stroke)
        .set_stroke_width(width);

    let mut builder = skia_safe::PathBuilder::new();

    skia_safe::path_utils::fill_path_with_paint(path, &paint, &mut builder, None, None);

    builder.detach()
}

// Consume final drawing coordinates; all math placement belongs to the layout engine.
fn emit(
    item: &DisplayItem,
    cache: &mut FormulaCache,
    color: Option<Color>,
    parts: &mut Vec<FormulaPart>,
) {
    match item {
        DisplayItem::GlyphPath {
            x,
            y,
            scale,
            font,
            char_code,
            ..
        } => {
            if let Some(path) = cache.glyph(font, *char_code) {
                let scale = *scale as f32 / 1000.0;

                parts.push(FormulaPart {
                    path: path
                        .with_transform(&skia_safe::Matrix::scale((scale, scale)))
                        .with_offset((*x as f32, *y as f32)),
                    color,
                    key: String::new(),
                    group: 0,
                });
            }
        }
        DisplayItem::Rect {
            x,
            y,
            width,
            height,
            ..
        } => {
            parts.push(FormulaPart {
                path: skia_safe::Path::rect(
                    skia_safe::Rect::from_xywh(*x as f32, *y as f32, *width as f32, *height as f32),
                    None,
                ),
                color,
                key: String::new(),
                group: 0,
            });
        }
        DisplayItem::Line {
            x,
            y,
            width,
            thickness,
            dashed,
            ..
        } => {
            if *thickness <= 0.0 || *width <= 0.0 {
                return;
            }

            let dash = if *dashed { 4.0 * thickness } else { *width };
            let mut offset = 0.0;

            while offset < *width {
                parts.push(FormulaPart {
                    path: skia_safe::Path::rect(
                        skia_safe::Rect::from_xywh(
                            (x + offset) as f32,
                            (y - thickness * 0.5) as f32,
                            dash.min(width - offset) as f32,
                            *thickness as f32,
                        ),
                        None,
                    ),
                    color,
                    key: String::new(),
                    group: 0,
                });

                offset += dash * 2.0;
            }
        }
        DisplayItem::Path {
            x,
            y,
            commands,
            fill,
            ..
        } => {
            let mut builder = skia_safe::PathBuilder::new();
            let mut flush = |builder: &mut skia_safe::PathBuilder| {
                let path = builder.detach();

                if !path.is_empty() {
                    let path = if *fill { path } else { stroked(&path, 0.04) };

                    parts.push(FormulaPart {
                        path: path.with_offset((*x as f32, *y as f32)),
                        color,
                        key: String::new(),
                        group: 0,
                    });
                }
            };

            for command in commands {
                match *command {
                    PathCommand::MoveTo { x, y } => {
                        // Independent filled contours may have opposing winding directions.
                        if *fill {
                            flush(&mut builder);
                        }

                        builder.move_to((x as f32, y as f32));
                    }
                    PathCommand::LineTo { x, y } => {
                        builder.line_to((x as f32, y as f32));
                    }
                    PathCommand::QuadTo { x1, y1, x, y } => {
                        builder.quad_to((x1 as f32, y1 as f32), (x as f32, y as f32));
                    }
                    PathCommand::CubicTo {
                        x1,
                        y1,
                        x2,
                        y2,
                        x,
                        y,
                    } => {
                        builder.cubic_to(
                            (x1 as f32, y1 as f32),
                            (x2 as f32, y2 as f32),
                            (x as f32, y as f32),
                        );
                    }
                    PathCommand::Close => {
                        builder.close();
                    }
                }
            }

            flush(&mut builder);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_word_groups_respect_spaces_and_symbols() {
        let parts = morph_geometry(r"\text{local d = wji make:func}", 0.0);
        let mut lengths = Vec::new();
        for (index, part) in parts.iter().enumerate() {
            if index == 0 || part.group != parts[index - 1].group {
                lengths.push(0);
            }
            *lengths.last_mut().unwrap() += 1;
        }
        assert_eq!(lengths, [5, 1, 1, 3, 4, 1, 4]);
    }

    #[test]
    fn grouped_morph_keeps_words_whole_inside_formula_roles() {
        use crate::core::objects::text_morph::morph_parts;
        for (from, to) in [
            ("abc+def", "def+abcd"),
            (r"\frac{abc}{def}", r"\frac{abcd}{def}"),
        ] {
            let from = morph_geometry(from, 0.0);
            let to = morph_geometry(to, 0.0);
            let draws = morph_parts(&from, &to, 0.5, true);
            assert!(
                draws
                    .iter()
                    .filter(|draw| !draw.incoming && draw.part.key.contains(":97/"))
                    .all(|draw| draw.opacity == 0.5)
            );
            assert!(
                draws
                    .iter()
                    .filter(|draw| !draw.incoming && draw.part.key.contains(":100/"))
                    .all(|draw| draw.opacity == 1.0)
            );
        }
    }

    #[test]
    fn morph_preserves_fraction_script_and_matrix_groups() {
        use crate::core::objects::text_morph::morph_parts;
        for (from, to, fading) in [
            (r"\frac{x}{y}", r"\frac{y}{x}", 4),
            (r"x^{a}_{b}", r"x^{b}_{a}", 4),
            (
                r"\begin{matrix} x & y \\ z & w \end{matrix}",
                r"\begin{matrix} y & x \\ w & z \end{matrix}",
                8,
            ),
            (r"\frac{\sqrt{x}}{x}", r"\frac{\sqrt{y}}{x}", 2),
        ] {
            let from = morph_geometry(from, 0.0);
            let to = morph_geometry(to, 0.0);
            let draws = morph_parts(&from, &to, 0.5, false);
            assert_eq!(
                draws.iter().filter(|draw| draw.opacity == 0.5).count(),
                fading
            );
        }
        let from = morph_geometry(r"\frac{x+x}{x}", 0.0);
        let to = morph_geometry(r"\frac{x+x+1}{x}", 0.0);
        let draws = morph_parts(&from, &to, 0.5, false);
        assert_eq!(
            draws.iter().filter(|draw| draw.opacity == 1.0).count(),
            from.len()
        );
        assert_eq!(draws.iter().filter(|draw| draw.opacity == 0.5).count(), 2);
    }

    #[test]
    fn adding_scripts_keeps_the_base_and_interpolates_explicit_colors() {
        use crate::core::objects::text_morph::morph_parts;
        let from = morph_geometry(r"\color{red}{x}", 0.0);
        let to = morph_geometry(r"\color{blue}{x}^2", 0.0);
        let draws = morph_parts(&from, &to, 0.5, false);
        let base = draws
            .iter()
            .find(|draw| draw.opacity == 1.0)
            .unwrap_or_else(|| {
                panic!(
                    "Matching base is missing: {:?} -> {:?}.",
                    from.iter().map(|part| &part.key).collect::<Vec<_>>(),
                    to.iter().map(|part| &part.key).collect::<Vec<_>>()
                )
            });
        assert_eq!(base.color(Color::WHITE), Color::new(0.5, 0.0, 0.5, 1.0));
        assert_eq!(draws.iter().filter(|draw| draw.opacity == 0.5).count(), 1);
    }

    fn colored_bounds(formula: &FormulaGeometry, color: Color) -> skia_safe::Rect {
        formula
            .parts
            .iter()
            .filter(|part| part.color == Some(color))
            .map(|part| part.path.compute_tight_bounds())
            .reduce(|a, b| {
                skia_safe::Rect::new(
                    a.left.min(b.left),
                    a.top.min(b.top),
                    a.right.max(b.right),
                    a.bottom.max(b.bottom),
                )
            })
            .expect("Colored formula parts are missing.")
    }

    #[test]
    fn nested_numerators_and_denominators_clear_the_fraction_bar() {
        for numerator in [
            r"\sqrt{x}",
            r"\sqrt{\frac{x+1}{y}}",
            r"\int_0^1 x^2\,dx",
            r"\sum_{i=0}^n i",
        ] {
            let text = format!(
                r"\frac{{\color{{red}}{{{numerator}}}}}{{\color{{blue}}{{\sqrt[3]{{y}}}}}}"
            );
            let formula = geometry(&text);
            let numerator = colored_bounds(&formula, Color::RED);
            let denominator = colored_bounds(&formula, Color::BLUE);
            let bar = formula
                .parts
                .iter()
                .find(|part| part.color.is_none())
                .unwrap()
                .path
                .compute_tight_bounds();

            assert!(
                numerator.bottom < bar.top,
                "Numerator crosses the fraction bar: {text}."
            );

            assert!(
                denominator.top > bar.bottom,
                "Denominator crosses the fraction bar: {text}."
            );
        }
    }

    #[test]
    fn large_operator_scripts_follow_the_operator_extents() {
        for operator in [r"\int", r"\sum", r"\prod"] {
            let formula = geometry(&format!(
                r"{operator}_{{\color{{blue}}{{0}}}}^{{\color{{red}}{{1}}}}"
            ));
            let operator = formula
                .parts
                .iter()
                .find(|part| part.color.is_none())
                .unwrap()
                .path
                .compute_tight_bounds();
            let upper = colored_bounds(&formula, Color::RED);
            let lower = colored_bounds(&formula, Color::BLUE);

            assert!(upper.center_y() < operator.top + operator.height() * 0.3);
            assert!(lower.center_y() > operator.bottom - operator.height() * 0.3);
        }
    }

    #[test]
    fn script_scales_and_explicit_colors_survive_geometry_caching() {
        let formula = geometry(r"x^{x^x}");
        let heights: Vec<_> = formula
            .parts
            .iter()
            .map(|part| part.path.compute_tight_bounds().height())
            .collect();

        assert_eq!(heights.len(), 3);
        assert!(heights[0] > heights[1] && heights[1] > heights[2]);
        assert!(Rc::ptr_eq(&formula, &geometry(r"x^{x^x}")));
        assert!(geometry("").parts.is_empty());

        let colors = geometry(r"x+\color{black}{y}+\color{white}{z}");

        assert!(colors.parts.iter().any(|part| part.color.is_none()));

        assert!(
            colors
                .parts
                .iter()
                .any(|part| part.color == Some(Color::BLACK))
        );

        assert!(
            colors
                .parts
                .iter()
                .any(|part| part.color == Some(Color::WHITE))
        );
    }

    #[test]
    #[ignore = "Manual visual layout review."]
    fn render_layout_gallery() {
        let expressions = [
            r"\frac{\sqrt{x^2+1}}{\sqrt[3]{y}}",
            r"\int_{0}^{1}\frac{\sqrt{x}}{1+x^2}\,dx",
            r"\int_{-\infty}^{\infty} e^{-x^2}\,dx = \sqrt{\pi}",
            r"\sum_{i=0}^{n} i^2 = \frac{n(n+1)(2n+1)}{6}",
            r"\sqrt{\frac{1+\sqrt{x}}{1+\frac{1}{y}}}",
            r"\left(\begin{matrix} \frac{\sqrt{x}}{y} & x^{x^x} \\ \int_0^1 f(x)\,dx & \sum_{i=1}^n i \end{matrix}\right)",
            r"\boxed{\color{red}{\frac{x^2}{\sqrt{2}}}}+\cancel{y}",
        ];
        let mut surface = skia_safe::surfaces::raster_n32_premul((1200, 1300)).unwrap();
        let canvas = surface.canvas();

        canvas.clear(skia_safe::Color::WHITE);

        for (row, text) in expressions.iter().enumerate() {
            let formula = geometry(text);
            let scale = 52.0_f32.min(1080.0 / formula.size.x);

            canvas.save();
            canvas.translate((600.0, 90.0 + row as f32 * 180.0));
            canvas.scale((scale, scale));

            for part in &formula.parts {
                let color = part.color.unwrap_or(Color::BLACK);
                let mut paint = skia_safe::Paint::default();

                paint.set_anti_alias(true).set_color4f(
                    skia_safe::Color4f::new(color.r, color.g, color.b, color.a),
                    None,
                );

                canvas.draw_path(&part.path, &paint);
            }

            canvas.restore();
        }

        let image = surface.image_snapshot();
        let data = image
            .encode(None, skia_safe::EncodedImageFormat::PNG, None)
            .unwrap();

        std::fs::write("/tmp/kinematic-latex-layout.png", data.as_bytes()).unwrap();
    }
}
