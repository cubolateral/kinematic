use kinematic::{hecs, prelude::*, skia_safe};

#[derive(Clone, Trackable)]
struct OffsetShape {
    #[track]
    offset: f32,
}

impl Default for OffsetShape {
    fn default() -> Self {
        Self { offset: 32.0 }
    }
}

#[derive(Object)]
#[object(spatial = "2d", builder = "offset_object")]
struct OffsetObject {
    #[trackable]
    shape: OffsetShape,
    #[trackable]
    style: Style,
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: Draw2D,
}

impl Default for OffsetObject {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            style: Default::default(),
            transform: Default::default(),
            draw: Draw2D {
                on_draw: |world, entity, canvas, opacity| {
                    let shape = world.get::<&OffsetShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let bounds = skia_safe::Rect::from_xywh(shape.offset, -8.0, 16.0, 16.0);
                    let [r, g, b, a] = style.fill.rgba();
                    let mut paint =
                        skia_safe::Paint::new(skia_safe::Color4f::new(r, g, b, a * opacity), None);
                    paint.set_anti_alias(true);
                    canvas.draw_rect(bounds, &paint);
                    let [r, g, b, a] = style.stroke.rgba();
                    paint
                        .set_style(skia_safe::PaintStyle::Stroke)
                        .set_stroke_width(style.stroke_width)
                        .set_color4f(skia_safe::Color4f::new(r, g, b, a * opacity), None);
                    canvas.draw_rect(bounds, &paint);
                },
                box_size: |_, _| vec2(16.0, 16.0),
                visual_bounds: |world, entity| {
                    let shape = world.get::<&OffsetShape>(entity).unwrap();
                    let style = world.get::<&Style>(entity).unwrap();
                    let padding = style.stroke_width.max(0.0) * 0.5;
                    skia_safe::Rect::new(
                        shape.offset - padding,
                        -8.0 - padding,
                        shape.offset + 16.0 + padding,
                        8.0 + padding,
                    )
                },
                ..Default::default()
            },
        }
    }
}

#[test]
fn custom_2d_object_morphs_without_dedicated_morph_code() {
    let mut scene = Scene::new();
    let source = offset_object()
        .offset(-48.0)
        .fill(Color::RED)
        .stroke(Color::WHITE)
        .stroke_width(4.0)
        .build(&mut scene);
    let target = offset_object()
        .offset(32.0)
        .fill(Color::BLUE)
        .stroke(Color::WHITE)
        .stroke_width(4.0)
        .build(&mut scene);
    scene.world_2d().add(&source);
    morph()
        .duration(1.0)
        .easing(Easing::Linear)
        .play(&source, &target);

    scene.update(0.5);
    let mut surface = skia_safe::surfaces::raster_n32_premul((160, 80)).unwrap();
    surface.canvas().translate((80.0, 40.0));
    scene.draw(surface.canvas());
    assert!((0..80).any(|y| (0..160).any(|x| {
        surface.peek_pixels().unwrap().get_color((x, y)).a() > 0
    })));

    scene.update(1.0);
    surface.canvas().clear(skia_safe::colors::TRANSPARENT);
    scene.draw(surface.canvas());
    assert_eq!(
        surface.peek_pixels().unwrap().get_color((120, 40)),
        skia_safe::Color::BLUE
    );
}
