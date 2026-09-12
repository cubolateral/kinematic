use kinematic_macros::{Object, Trackable};

use crate::core::{
    Tween,
    components::{
        Draw2D, Morph, Style, Transform2D, draw_complete_styled_path, stroke_width_for_scale,
    },
    objects::{
        CreationDraw, ObjectHandler,
        latex_geometry::geometry,
        particle::Silhouette,
        particle_visual_key,
        string_morph::{ContentMorph, ContentMorphTransition, fade_string, morph_string},
        text_2d::weighted_path,
    },
    types::Vector2,
};

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
/// ```
/// use kinematic::prelude::*;
///
/// let mut scene = Scene::new();
/// let formula = latex_2d()
///     .text(r"\frac{1}{2}")
///     .size(64.0)
///     .build(&mut scene);
/// scene.get_world_2d().add(&formula);
/// formula.morph(r"\sqrt{2}").duration(2.0).play();
/// ```
#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "latex_2d")]
#[morph]
pub struct Latex2D {
    #[trackable]
    pub shape: Latex2DShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

fn latex_box(shape: &Latex2DShape) -> Vector2 {
    geometry(&shape.text).size * shape.size.max(0.0) + Vector2::splat(shape.thickness.max(0.0))
}

fn latex_morph_silhouette(
    shape: &Latex2DShape,
    style: &Style,
    transform: &Transform2D,
) -> Silhouette {
    let size = latex_box(shape);
    let padding = stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale) * 0.5 + 2.0;
    let bounds = skia_safe::Rect::new(
        -size.x * 0.5 - padding,
        -size.y * 0.5 - padding,
        size.x * 0.5 + padding,
        size.y * 0.5 + padding,
    );
    Silhouette::capture(bounds, |canvas| {
        draw_complete_latex(shape, style, 1.0, transform.scale, canvas);
    })
}

fn draw_latex_morph(
    transition: &ContentMorphTransition,
    shape: &Latex2DShape,
    style: &Style,
    transform: &Transform2D,
    progress: f32,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let shape_for = |text: &str| {
        let mut shape = shape.clone();
        shape.text = text.to_owned();
        shape
    };
    transition.draw(canvas, progress, opacity, |text, opacity| {
        draw_complete_latex(&shape_for(text), style, opacity, transform.scale, canvas);
    });
}

fn draw_complete_latex(
    shape: &Latex2DShape,
    style: &Style,
    opacity: f32,
    scale: Vector2,
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

fn draw_latex(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas, opacity: f32) {
    let shape = world.get::<&Latex2DShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
    let morph_state = world.get::<&Morph>(entity).unwrap();
    let transform = world.get::<&Transform2D>(entity).unwrap();

    if let Ok(morph) = world.get::<&ContentMorph>(entity)
        && morph.active
        && morph.progress > 0.0
    {
        draw_latex_morph(
            &morph.transitions[morph.transition as usize],
            &shape,
            &style,
            &transform,
            morph.progress.clamp(0.0, 1.0),
            opacity,
            canvas,
        );
        return;
    }

    if morph_state.particles_enabled && morph_state.progress < 1.0 {
        let size = latex_box(&shape);
        let stroke_padding =
            stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale) * 0.5;
        let bounds = skia_safe::Rect::new(
            -size.x * 0.5 - stroke_padding,
            -size.y * 0.5 - stroke_padding,
            size.x * 0.5 + stroke_padding,
            size.y * 0.5 + stroke_padding,
        );
        let visual_key = particle_visual_key(
            "Latex2D",
            &style,
            &[shape.size, transform.scale.x, transform.scale.y],
            &[&shape.text],
        );

        if (CreationDraw {
            entity,
            cache_slot: 0,
            bounds,
            visual_key,
            style: &style,
            pixel_color: None,
            morph: &morph_state,
            opacity,
            canvas,
        })
        .render(|target, target_opacity| {
            draw_complete_latex(&shape, &style, target_opacity, transform.scale, target);
        }) {
            return;
        }
    }

    draw_complete_latex(&shape, &style, opacity, transform.scale, canvas);
}

impl Default for Latex2D {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: draw_latex,
                get_box: |world, entity| latex_box(&world.get::<&Latex2DShape>(entity).unwrap()),
                ..Default::default()
            },
        }
    }
}

impl Latex2DHandler {
    /// Cross-fades the current formula source into `text` on the same object.
    pub fn fade(&self, text: impl Into<String>) -> Tween<Latex2D> {
        let from = self.get(Latex2DShape::text_property());
        let to = text.into();
        let tween = self.text(to.clone());
        fade_string(tween, self.get_id(), from, to)
    }

    /// Morphs this formula into `text` through particle silhouettes.
    ///
    /// Unlike [`crate::core::effects::morph`], this keeps the same formula object and
    /// changes its discrete string value when the returned tween completes.
    pub fn morph(&self, text: impl Into<String>) -> Tween<Latex2D> {
        let text = text.into();
        let from_text = self.get(Latex2DShape::text_property());
        let tween = self.text(text.clone());
        morph_string(
            tween,
            self.get_id(),
            from_text,
            text,
            latex_morph_silhouette,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

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
    fn fade_swaps_formula_halfway_without_creating_another_object() {
        struct FadingFormula;

        impl SceneBuilder for FadingFormula {
            fn build(&mut self, scene: &mut Scene) {
                let formula = latex_2d().text("x").build(scene);
                scene.get_world_2d().add(&formula);
                formula
                    .fade(r"\frac{1}{2}")
                    .duration(2.0)
                    .easing(Easing::Linear)
                    .play();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut FadingFormula), 2.0);
        assert_eq!(scene.get_world().query::<&Latex2DShape>().iter().count(), 1);

        let state = |scene: &Scene| {
            let world = scene.get_world();
            let mut query = world.query::<(&Latex2DShape, &Draw2D)>();
            let (shape, draw) = query.iter().next().unwrap();
            (shape.text.clone(), draw.opacity)
        };

        scene.update(0.5);
        assert_eq!(state(&scene).0, "x");
        scene.update(1.0);
        assert_eq!(state(&scene).0, "x");
        scene.update(1.5);
        assert_eq!(state(&scene).0, "x");
        scene.update(2.0);
        assert_eq!(state(&scene).0, r"\frac{1}{2}");
    }

    #[test]
    fn formula_creation_and_consecutive_morphs_are_seekable() {
        struct FormulaScene;
        impl SceneBuilder for FormulaScene {
            fn build(&mut self, scene: &mut Scene) {
                let formula = latex_2d().text(r"\frac{1}{2}").build(scene);
                assert_eq!(formula.get_name(), "Latex2D");
                scene.get_world_2d().add(&formula);
                creation().duration(1.0).play(&formula);
                formula.morph(r"\sqrt{2}").play();
                formula.morph(r"e^{i\pi}+1=0").play();
            }
        }
        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut FormulaScene), 3.0);
        assert_eq!(scene.get_world().len(), 4);
        scene.update(0.0);
        assert!(pixels(&scene).iter().all(|color| color.a() == 0));
        scene.update(0.5);
        assert!(pixels(&scene).iter().any(|color| color.a() > 0));
        scene.update(1.5);
        let first_morph = pixels(&scene);
        assert!(first_morph.iter().any(|color| color.a() > 0));
        assert_eq!(first_morph, pixels(&scene));
        scene.update(1.9999);
        let boundary = pixels(&scene);
        scene.update(2.0);
        assert_eq!(boundary, pixels(&scene));
        scene.update(2.5);
        {
            let world = scene.get_world();
            let mut query = world.query::<(&Latex2DShape, &ContentMorph)>();
            let (shape, morph) = query.iter().next().unwrap();
            assert_eq!(shape.text, r"\sqrt{2}");
            assert_eq!(morph.transition, 1);
            assert_eq!(morph.progress, 0.5);
        }
        scene.update(3.0);
        scene.update(1.5);
        assert_eq!(first_morph, pixels(&scene));
        scene.update(3.0);
        let world = scene.get_world();
        assert_eq!(
            world.query::<&Latex2DShape>().iter().next().unwrap().text,
            r"e^{i\pi}+1=0"
        );
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
        scene.get_world_2d().add(&formula);
        let bounds = formula.get_box();
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
        let doubled = formula.get_box();
        assert!((doubled.x - bounds.x * 2.0).abs() < 0.001);
        assert!((doubled.y - bounds.y * 2.0).abs() < 0.001);
    }
}
