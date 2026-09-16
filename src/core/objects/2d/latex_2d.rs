use kinematic_macros::{Object, Trackable};

use crate::core::{
    Tween,
    components::{Draw2D, Style, Transform2D, draw_complete_styled_path, stroke_width_for_scale},
    objects::{
        ObjectHandler,
        latex_geometry::{FormulaGlyph, geometry},
        particle::{ParticleTransform, Silhouette, morph_opacities},
        string_morph::{
            ContentMorph, ContentMorphTransition, MorphPath, MovingMorphPath, PathMorphPlan,
            PreparedContentMorph, fade_string, match_items, morph_string_with,
        },
        text_2d::weighted_path,
    },
    types::{Color, Vector2},
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
/// scene.world_2d().add(&formula);
/// formula.morph(r"\sqrt{2}").duration(2.0).play();
/// ```
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
}

fn latex_box(shape: &Latex2DShape) -> Vector2 {
    geometry(&shape.text).size * shape.size.max(0.0) + Vector2::splat(shape.thickness.max(0.0))
}

fn morph_parts(shape: &Latex2DShape) -> Vec<(Option<FormulaGlyph>, MorphPath)> {
    let scale = shape.size.max(0.0);
    geometry(&shape.text)
        .parts
        .iter()
        .map(|part| {
            (
                part.glyph.clone(),
                MorphPath {
                    path: std::sync::Arc::new(std::sync::Mutex::new(
                        part.path
                            .with_transform(&skia_safe::Matrix::scale((scale, scale))),
                    )),
                    color: part.color,
                },
            )
        })
        .collect()
}

fn draw_morph_parts(
    parts: &[MorphPath],
    thickness: f32,
    style: &Style,
    transform: &Transform2D,
    opacity: f32,
    canvas: &skia_safe::Canvas,
) {
    let mut part_style = style.clone();
    for part in parts {
        part_style.fill = part.color.unwrap_or(style.fill);
        draw_complete_styled_path(
            &weighted_path(&part.path.lock().unwrap(), thickness),
            &part_style,
            transform.scale,
            opacity,
            canvas,
        );
    }
}

fn morph_silhouette(
    parts: &[MorphPath],
    shape: &Latex2DShape,
    style: &Style,
    transform: &Transform2D,
) -> Silhouette {
    let mut bounds: Option<skia_safe::Rect> = None;
    for part in parts {
        let path = weighted_path(&part.path.lock().unwrap(), shape.thickness.max(0.0));
        let part_bounds = path.compute_tight_bounds();
        match &mut bounds {
            Some(bounds) => bounds.join(part_bounds),
            None => bounds = Some(part_bounds),
        }
    }
    let mut bounds = bounds.unwrap_or_default();
    let padding = stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale) * 0.5 + 2.0;
    bounds.outset((padding, padding));
    Silhouette::capture(bounds, |canvas| {
        draw_morph_parts(
            parts,
            shape.thickness.max(0.0),
            style,
            transform,
            1.0,
            canvas,
        );
    })
}

fn prepare_latex_morph(
    from_shape: &Latex2DShape,
    from_style: &Style,
    from_transform: &Transform2D,
    to_shape: &Latex2DShape,
    to_style: &Style,
    to_transform: &Transform2D,
) -> PathMorphPlan {
    let from = morph_parts(from_shape);
    let to = morph_parts(to_shape);
    let from_glyphs: Vec<_> = from
        .iter()
        .enumerate()
        .filter_map(|(index, (glyph, part))| {
            glyph.as_ref().map(|glyph| {
                let center = part.path.lock().unwrap().compute_tight_bounds().center();
                (index, glyph, Vector2::new(center.x, center.y))
            })
        })
        .collect();
    let to_glyphs: Vec<_> = to
        .iter()
        .enumerate()
        .filter_map(|(index, (glyph, part))| {
            glyph.as_ref().map(|glyph| {
                let center = part.path.lock().unwrap().compute_tight_bounds().center();
                (index, glyph, Vector2::new(center.x, center.y))
            })
        })
        .collect();
    let matches = match_items(
        &from_glyphs
            .iter()
            .map(|(_, glyph, _)| *glyph)
            .collect::<Vec<_>>(),
        &from_glyphs
            .iter()
            .map(|(_, _, origin)| *origin)
            .collect::<Vec<_>>(),
        &to_glyphs
            .iter()
            .map(|(_, glyph, _)| *glyph)
            .collect::<Vec<_>>(),
        &to_glyphs
            .iter()
            .map(|(_, _, origin)| *origin)
            .collect::<Vec<_>>(),
    );
    let mut matched_from = vec![false; from.len()];
    let mut matched_to = vec![false; to.len()];
    let mut stable = Vec::with_capacity(matches.len());
    let mut from_anchors = Vec::with_capacity(matches.len());
    let mut to_anchors = Vec::with_capacity(matches.len());

    for (from_match, to_match) in matches {
        let from_index = from_glyphs[from_match].0;
        let to_index = to_glyphs[to_match].0;
        matched_from[from_index] = true;
        matched_to[to_index] = true;
        from_anchors.push(from_glyphs[from_match].2);
        to_anchors.push(to_glyphs[to_match].2);
        stable.push(MovingMorphPath {
            from: from[from_index].1.clone(),
            to: to[to_index].1.clone(),
        });
    }

    let source: Vec<_> = from
        .into_iter()
        .enumerate()
        .filter_map(|(index, (_, part))| (!matched_from[index]).then_some(part))
        .collect();
    let target: Vec<_> = to
        .into_iter()
        .enumerate()
        .filter_map(|(index, (_, part))| (!matched_to[index]).then_some(part))
        .collect();
    let mut from_silhouette = morph_silhouette(&source, from_shape, from_style, from_transform);
    let mut to_silhouette = morph_silhouette(&target, to_shape, to_style, to_transform);
    if from_silhouette.is_empty() && !to_silhouette.is_empty() {
        from_silhouette = to_silhouette.collapsed_at(&from_anchors);
    } else if to_silhouette.is_empty() && !from_silhouette.is_empty() {
        to_silhouette = from_silhouette.collapsed_at(&to_anchors);
    }

    PathMorphPlan {
        stable,
        source,
        target,
        particles: ParticleTransform::sampled(
            from_silhouette,
            to_silhouette,
            crate::core::Easing::Linear,
        ),
    }
}

fn mix_color(from: Color, to: Color, progress: f32) -> Color {
    Color::new(
        from.r + (to.r - from.r) * progress,
        from.g + (to.g - from.g) * progress,
        from.b + (to.b - from.b) * progress,
        from.a + (to.a - from.a) * progress,
    )
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
    if transition.is_fade() {
        let shape_for = |text: &str| {
            let mut shape = shape.clone();
            shape.text = text.to_owned();
            shape
        };
        transition.draw(progress, opacity, |text, opacity| {
            draw_latex_2d(&shape_for(text), style, transform.scale, opacity, canvas);
        });
        return;
    }
    let plan = transition.path_plan();
    let (source_opacity, target_opacity) = morph_opacities(progress);
    draw_morph_parts(
        &plan.source,
        shape.thickness.max(0.0),
        style,
        transform,
        opacity * source_opacity,
        canvas,
    );
    draw_morph_parts(
        &plan.target,
        shape.thickness.max(0.0),
        style,
        transform,
        opacity * target_opacity,
        canvas,
    );
    plan.particles.draw(canvas, progress, opacity);

    let movement = progress * progress * (3.0 - 2.0 * progress);
    let mut part_style = style.clone();
    for part in &plan.stable {
        let from = part.from.path.lock().unwrap();
        let to = part.to.path.lock().unwrap();
        let path = from
            .interpolate(&to, 1.0 - movement)
            .expect("Matching LaTeX glyph paths must be interpolatable.");
        part_style.fill = mix_color(
            part.from.color.unwrap_or(style.fill),
            part.to.color.unwrap_or(style.fill),
            movement,
        );
        draw_complete_styled_path(
            &weighted_path(&path, shape.thickness.max(0.0)),
            &part_style,
            transform.scale,
            opacity,
            canvas,
        );
    }
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

pub(crate) fn draw_latex_effect(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) -> bool {
    let shape = world.get::<&Latex2DShape>(entity).unwrap();
    let style = world.get::<&Style>(entity).unwrap();
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
        return true;
    }

    false
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
    draw_latex_2d(&shape, &style, transform.scale, opacity, canvas);
}

impl Default for Latex2D {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: draw_latex_object,
                box_size: |world, entity| latex_box(&world.get::<&Latex2DShape>(entity).unwrap()),
                visual_bounds: |world, entity| {
                    let shape = world.get::<&Latex2DShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let size = latex_box(&shape);
                    crate::core::components::styled_bounds(
                        skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y),
                        &style,
                        transform.scale,
                    )
                },
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
        fade_string(tween, self.entity(), from, to)
    }

    /// Morphs this formula into `text` through particle silhouettes.
    ///
    /// Unlike [`crate::core::effects::morph`], this keeps the same formula object and
    /// changes its discrete string value when the returned tween completes.
    pub fn morph(&self, text: impl Into<String>) -> Tween<Latex2D> {
        let text = text.into();
        let from_text = self.get(Latex2DShape::text_property());
        let tween = self.text(text.clone());
        morph_string_with(
            tween,
            self.entity(),
            from_text,
            text,
            |from_shape, from_style, from_transform, to_shape, to_style, to_transform| {
                PreparedContentMorph::Paths(prepare_latex_morph(
                    &from_shape,
                    &from_style,
                    &from_transform,
                    &to_shape,
                    &to_style,
                    &to_transform,
                ))
            },
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
                scene.world_2d().add(&formula);
                formula
                    .fade(r"\frac{1}{2}")
                    .duration(2.0)
                    .easing(Easing::Linear)
                    .play();
            }
        }

        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut FadingFormula), 2.0);
        assert_eq!(scene.world().query::<&Latex2DShape>().iter().count(), 1);

        let state = |scene: &Scene| {
            let world = scene.world();
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
                assert_eq!(formula.name(), "Latex2D");
                scene.world_2d().add(&formula);
                creation().duration(1.0).play(&formula);
                formula.morph(r"\sqrt{2}").play();
                formula.morph(r"e^{i\pi}+1=0").play();
            }
        }
        let mut scene = Scene::new();
        assert_eq!(scene.build(&mut FormulaScene), 3.0);
        assert_eq!(scene.world().len(), 4);
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
            let world = scene.world();
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
        let world = scene.world();
        assert_eq!(
            world.query::<&Latex2DShape>().iter().next().unwrap().text,
            r"e^{i\pi}+1=0"
        );
    }

    #[test]
    fn morph_keeps_shared_formula_glyphs_out_of_particle_silhouettes() {
        let from = Latex2DShape {
            text: "x+1".to_owned(),
            ..Default::default()
        };
        let to = Latex2DShape {
            text: "x+2".to_owned(),
            ..Default::default()
        };
        let plan = prepare_latex_morph(
            &from,
            &Style::default(),
            &Transform2D::default(),
            &to,
            &Style::default(),
            &Transform2D::default(),
        );

        assert_eq!(plan.stable.len(), 2);
        assert_eq!(plan.source.len(), 1);
        assert_eq!(plan.target.len(), 1);
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
