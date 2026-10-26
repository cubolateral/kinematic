use crate::core::{
    components::{Draw2D, Filter, Style, Transform2D, draw_complete_styled_path},
    objects::{TextFade, latex_geometry::geometry, text_2d::weighted_path},
    types::Vector2,
};
use kinematic_macros::{Object, Trackable};

/// Mathematical source and size of a LaTeX object.
#[derive(Clone, Trackable)]
pub struct Latex2DShape {
    /// LaTeX math source, without dollar delimiters.
    #[track]
    pub text: String,
    /// Font size in logical canvas units.
    #[track(min = 0.0)]
    pub size: f32,
    /// Extra glyph thickness in logical canvas units.
    #[track(min = 0.0)]
    pub thickness: f32,
}

impl Default for Latex2DShape {
    fn default() -> Self {
        Self {
            text: r"e^{i\pi}+1=0".to_owned(),
            size: 64.0,
            thickness: 0.0,
        }
    }
}

/// Built-in LaTeX math scene object, rendered with embedded KaTeX fonts.
///
/// Supports mathematical LaTeX accepted by RaTeX in display style.
/// Invalid or unsupported source panics when its geometry is first requested.
///
#[derive(Object)]
#[object(spatial = "2d", builder = "latex_2d")]
pub struct Latex2D {
    #[trackable]
    pub shape: Latex2DShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
    #[trackable]
    pub filter: Filter,

    pub(crate) fade: TextFade,
}

fn latex_box(shape: &Latex2DShape) -> Vector2 {
    geometry(&shape.text).size * shape.size.max(0.0) + Vector2::splat(shape.thickness.max(0.0))
}

fn latex_object_box(world: &hecs::World, entity: hecs::Entity) -> Vector2 {
    let shape = world.get::<&Latex2DShape>(entity).unwrap();
    let fade = world.get::<&TextFade>(entity).unwrap();
    let size = latex_box(&shape);

    fade.incoming_text(&shape.text).map_or(size, |text| {
        let incoming = latex_box(&Latex2DShape {
            text: text.to_owned(),
            ..(*shape).clone()
        });

        Vector2::new(size.x.max(incoming.x), size.y.max(incoming.y))
    })
}

/// Draws a LaTeX formula in local 2D coordinates from reusable rendering data.
pub fn draw_latex_2d(
    shape: &Latex2DShape,
    style: &Style,
    scale: Vector2,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let geometry = geometry(&shape.text);
    let size = shape.size.max(0.0);

    if size <= 0.0 {
        return;
    }

    canvas.save();
    canvas.scale((size, size));

    let mut part_style = style.clone();

    part_style.stroke_width /= size;

    for part in &geometry.parts {
        part_style.fill = part.color.unwrap_or(style.fill);

        let path = weighted_path(&part.path, shape.thickness / size);

        draw_complete_styled_path(&path, &part_style, scale, opacity, canvas);
    }

    canvas.restore();
}

fn draw_latex_object(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) {
    let shape = world.get::<&Latex2DShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
    let transform = world.get::<&Transform2D>(entity).unwrap();

    let fade = world.get::<&TextFade>(entity).unwrap();

    if let Some(text) = fade.incoming_text(&shape.text) {
        if fade.morph {
            use crate::core::objects::{latex_geometry::morph_geometry, text_morph::morph_parts};
            let size = shape.size.max(0.0);
            if size <= 0.0 {
                return;
            }
            let from = morph_geometry(&shape.text, shape.thickness / size);
            let to = morph_geometry(text, shape.thickness / size);
            canvas.save();
            canvas.scale((size, size));
            let mut part_style = (*style).clone();
            part_style.stroke_width /= size;
            for part in morph_parts(&from, &to, fade.progress, fade.group) {
                part_style.fill = part.color(style.fill);
                draw_complete_styled_path(
                    &part.path(),
                    &part_style,
                    transform.scale,
                    opacity * part.opacity,
                    canvas,
                );
            }
            canvas.restore();
            return;
        }
        let incoming = Latex2DShape {
            text: text.to_owned(),
            ..(*shape).clone()
        };

        draw_latex_2d(
            &shape,
            &style,
            transform.scale,
            opacity * (1.0 - fade.progress),
            canvas,
        );
        draw_latex_2d(
            &incoming,
            &style,
            transform.scale,
            opacity * fade.progress,
            canvas,
        );
    } else {
        draw_latex_2d(&shape, &style, transform.scale, opacity, canvas);
    }
}

impl Default for Latex2D {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: draw_latex_object,
                box_size: latex_object_box,
                visual_bounds: |world, entity| {
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let size = latex_object_box(world, entity);

                    crate::core::components::styled_bounds(
                        skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y),
                        &style,
                        transform.scale,
                    )
                },
                ..Default::default()
            },
            filter: Default::default(),
            fade: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn pixels(scene: &Scene) -> Vec<skia_safe::Color> {
        let mut surface = skia_safe::surfaces::raster_n32_premul((640, 320)).unwrap();

        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((320.0, 160.0));
        scene.draw(surface.canvas());

        let pixels = surface.peek_pixels().unwrap();

        (0..320)
            .flat_map(|y| (0..640).map(move |x| (x, y)))
            .map(|point| pixels.get_color(point))
            .collect()
    }

    #[test]
    fn formulas_draw_decorations_colors_and_scale_within_their_bounds() {
        let mut scene = Scene::new();
        let formula = latex_2d()
            .text(r"\boxed{\color{red}{\frac{x^2}{\sqrt{2}}}}+\cancel{y}")
            .size(48.0)
            .fill(Color::BLUE)
            .build(&mut scene);

        assert!(pixels(&scene).iter().all(|color| color.a() == 0));
        scene.world_2d().add(&formula);

        let bounds = formula.box_size();
        let rendered = pixels(&scene);

        assert!(
            rendered
                .iter()
                .any(|color| color.r() > 200 && color.b() < 20)
        );

        assert!(
            rendered
                .iter()
                .any(|color| color.b() > 200 && color.r() < 20)
        );

        for (index, color) in rendered
            .iter()
            .enumerate()
            .filter(|(_, color)| color.a() > 0)
        {
            let x = (index % 640) as f32 - 320.0;
            let y = (index / 640) as f32 - 160.0;

            assert!(
                x.abs() <= bounds.x * 0.5 + 1.0 && y.abs() <= bounds.y * 0.5 + 1.0,
                "Formula ink exceeds its bounds: {color:?}."
            );
        }

        formula.size(96.0).play();

        let doubled = formula.box_size();

        assert!((doubled.x - bounds.x * 2.0).abs() < 0.001);
        assert!((doubled.y - bounds.y * 2.0).abs() < 0.001);
    }
}
