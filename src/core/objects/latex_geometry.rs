use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ratex_font::{FontId, katex_ttf_glyph_char};
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_types::{display_item::DisplayItem, path_command::PathCommand};

use crate::core::types::{Color, Vector2};

pub(super) struct FormulaPart {
    pub path: skia_safe::Path,
    pub color: Option<Color>,
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
        let display = to_display_list(&layout(&ast, &options));
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
        for (item, alternate) in display.items.iter().zip(&alternate.items) {
            let tint = item_color(item);
            let fill =
                (tint == item_color(alternate)).then(|| Color::new(tint.r, tint.g, tint.b, tint.a));
            emit(item, cache, fill, &mut geometry.parts);
        }
        let offset = -geometry.size * 0.5;
        for part in &mut geometry.parts {
            part.path = part.path.with_offset((offset.x, offset.y));
            // Include glyph overhangs in selection and particle capture bounds.
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
