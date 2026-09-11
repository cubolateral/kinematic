use kinematic_macros::{Object, Trackable};

use crate::core::{
    components::{
        Draw2D, Morph, Style, Transform2D, draw_complete_styled_path, draw_styled_path,
        stroke_width_for_scale,
    },
    objects::{CreationDraw, particle_visual_key},
    types::{Vector2, vec2},
};

#[derive(Clone, Trackable)]
/// Geometry of an elliptical object.
pub struct EllipseShape {
    #[track]
    pub radius: Vector2,
}

impl Default for EllipseShape {
    fn default() -> Self {
        Self {
            radius: vec2(128.0, 64.0),
        }
    }
}

#[derive(Object, hecs::Bundle)]
#[object(spatial = "2d", builder = "ellipse")]
#[morph]
/// Built-in elliptical scene object.
pub struct Ellipse {
    #[trackable]
    pub shape: EllipseShape,
    #[trackable]
    pub style: Style,
    #[trackable]
    pub transform: Transform2D,
    #[trackable]
    pub draw: Draw2D,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&EllipseShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let morph = world.get::<&Morph>(entity).unwrap();
                    let transform = world.get::<&Transform2D>(entity).unwrap();
                    let radius = shape.radius.abs();
                    let bounds = skia_safe::Rect::new(-radius.x, -radius.y, radius.x, radius.y);
                    let path = skia_safe::Path::oval(bounds, None);

                    if morph.particles_enabled && morph.progress < 1.0 {
                        let stroke_padding =
                            stroke_width_for_scale(style.stroke_width.max(0.0), transform.scale)
                                * 0.5;
                        let particle_bounds = skia_safe::Rect::new(
                            bounds.left - stroke_padding,
                            bounds.top - stroke_padding,
                            bounds.right + stroke_padding,
                            bounds.bottom + stroke_padding,
                        );
                        let visual_key = particle_visual_key(
                            "Ellipse",
                            &style,
                            &[radius.x, radius.y, transform.scale.x, transform.scale.y],
                            &[],
                        );

                        if (CreationDraw {
                            entity,
                            cache_slot: 0,
                            bounds: particle_bounds,
                            visual_key,
                            style: &style,
                            pixel_color: None,
                            morph: &morph,
                            opacity,
                            canvas,
                        })
                        .render(|target, target_opacity| {
                            draw_complete_styled_path(
                                &path,
                                &style,
                                transform.scale,
                                target_opacity,
                                target,
                            );
                        }) {
                            return;
                        }
                    }

                    draw_styled_path(&path, &style, transform.scale, opacity, canvas);
                },
                get_box: |world, entity| {
                    world.get::<&EllipseShape>(entity).unwrap().radius.abs() * 2.0
                },
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Scene, objects::*, types::vec2};

    #[test]
    fn ellipse_builder_sets_trackable_radii() {
        let mut scene = Scene::new();
        let ellipse = ellipse().radius(vec2(160.0, 80.0)).build(&mut scene);

        assert_eq!(ellipse.get_box(), vec2(320.0, 160.0));
        assert_eq!(
            ellipse.get(EllipseShape::radius_property()),
            vec2(160.0, 80.0)
        );
    }
}
