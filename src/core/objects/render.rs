use crate::core::{
    components::{Draw, Node, Transform},
    objects::{CameraTransform, children},
    types::Vector2,
};

pub(crate) fn active_camera_matrix(
    world: &hecs::World,
    root: hecs::Entity,
) -> Option<skia_safe::Matrix> {
    let mut camera = None;
    find_active_camera(world, root, &skia_safe::Matrix::new_identity(), &mut camera);
    camera
}

fn find_active_camera(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: &skia_safe::Matrix,
    camera: &mut Option<skia_safe::Matrix>,
) {
    let node = world
        .get::<&Node>(entity)
        .expect("Camera traversal object must contain a Node component.");
    if !node.is_activated {
        return;
    }

    let local = world
        .get::<&CameraTransform>(entity)
        .map(|transform| camera_matrix(&transform))
        .or_else(|_| {
            world
                .get::<&Transform>(entity)
                .map(|transform| transform_matrix(&transform))
        })
        .unwrap_or_else(|_| skia_safe::Matrix::new_identity());
    let global = skia_safe::Matrix::concat(parent, &local);

    if world.get::<&CameraTransform>(entity).is_ok() && global.invert().is_some() {
        *camera = Some(global.clone());
    }

    for child in children(world, entity) {
        find_active_camera(world, child, &global, camera);
    }
}

pub(crate) fn draw_entity(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    let node = world
        .get::<&Node>(entity)
        .expect("Drawn object must contain a Node component.");
    if !node.is_activated {
        return;
    }

    let draw = world
        .get::<&Draw>(entity)
        .expect("Drawn object must contain a Draw component.");
    let opacity = draw.opacity.clamp(0.0, 1.0);
    if opacity <= 0.0 {
        return;
    }

    let children = node.children.clone().unwrap_or_default();
    let save_count = canvas.save();
    apply_transform(world, entity, canvas);

    if children.is_empty() || opacity >= 1.0 {
        (draw.on_draw)(world, entity, canvas, opacity);

        for child in children {
            draw_entity(world, child, canvas);
        }
    } else {
        let layer_count = canvas.save_layer_alpha_f(None, opacity);
        (draw.on_draw)(world, entity, canvas, 1.0);

        for child in children {
            draw_entity(world, child, canvas);
        }

        canvas.restore_to_count(layer_count);
    }

    canvas.restore_to_count(save_count);
}

pub(crate) fn draw_entity_outline(
    world: &hecs::World,
    entity: hecs::Entity,
    target: hecs::Entity,
    canvas: &skia_safe::Canvas,
) -> bool {
    let node = world
        .get::<&Node>(entity)
        .expect("Outlined object must contain a Node component.");
    if !node.is_activated {
        return false;
    }

    let save_count = canvas.save();
    apply_transform(world, entity, canvas);

    let found = if entity == target {
        if let Some(bounds) = local_bounds(world, entity) {
            draw_outline(bounds, canvas);
        }

        true
    } else {
        children(world, entity)
            .into_iter()
            .any(|child| draw_entity_outline(world, child, target, canvas))
    };

    canvas.restore_to_count(save_count);
    found
}

pub(crate) fn pick_entity(
    world: &hecs::World,
    entity: hecs::Entity,
    point: Vector2,
) -> Option<hecs::Entity> {
    let node = world
        .get::<&Node>(entity)
        .expect("Picked object must contain a Node component.");
    let draw = world
        .get::<&Draw>(entity)
        .expect("Picked object must contain a Draw component.");

    if !node.is_activated || draw.opacity <= 0.0 {
        return None;
    }

    let point = inverse_transform_point(world, entity, point)?;

    if let Some(child) = node
        .children
        .as_ref()
        .into_iter()
        .flatten()
        .rev()
        .copied()
        .find_map(|child| pick_entity(world, child, point))
    {
        return Some(child);
    }

    let bounds = local_bounds(world, entity)?;

    (point.x >= bounds.left
        && point.x <= bounds.right
        && point.y >= bounds.top
        && point.y <= bounds.bottom)
        .then_some(entity)
}

#[doc(hidden)]
pub fn object_box(world: &hecs::World, entity: hecs::Entity) -> Vector2 {
    local_bounds(world, entity)
        .map(|bounds| Vector2::new(bounds.width(), bounds.height()))
        .unwrap_or(Vector2::ZERO)
}

fn local_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    let draw = world
        .get::<&Draw>(entity)
        .expect("Bounded object must contain a Draw component.");
    let size = (draw.get_box)(world, entity);
    let own = (size.x > 0.0 && size.y > 0.0)
        .then(|| skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y));
    let child_bounds = children(world, entity)
        .into_iter()
        .filter(|child| {
            world
                .get::<&Node>(*child)
                .map(|node| node.is_activated)
                .unwrap_or(false)
        })
        .filter_map(|child| transformed_bounds(world, child))
        .reduce(union_bounds);

    match (own, child_bounds) {
        (Some(own), Some(children)) => Some(union_bounds(own, children)),
        (Some(own), None) => Some(own),
        (None, children) => children,
    }
}

fn transformed_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    let bounds = local_bounds(world, entity)?;
    let Ok(transform) = world.get::<&Transform>(entity) else {
        return Some(bounds);
    };
    let sin = transform.rotation.sin();
    let cos = transform.rotation.cos();
    let points = [
        Vector2::new(bounds.left, bounds.top),
        Vector2::new(bounds.right, bounds.top),
        Vector2::new(bounds.right, bounds.bottom),
        Vector2::new(bounds.left, bounds.bottom),
    ]
    .map(|point| {
        let scaled = point * transform.scale;

        Vector2::new(
            scaled.x * cos - scaled.y * sin,
            scaled.x * sin + scaled.y * cos,
        ) + transform.position
    });

    Some(skia_safe::Rect::new(
        points
            .iter()
            .map(|point| point.x)
            .fold(f32::INFINITY, f32::min),
        points
            .iter()
            .map(|point| point.y)
            .fold(f32::INFINITY, f32::min),
        points
            .iter()
            .map(|point| point.x)
            .fold(f32::NEG_INFINITY, f32::max),
        points
            .iter()
            .map(|point| point.y)
            .fold(f32::NEG_INFINITY, f32::max),
    ))
}

fn inverse_transform_point(
    world: &hecs::World,
    entity: hecs::Entity,
    point: Vector2,
) -> Option<Vector2> {
    let Ok(transform) = world.get::<&Transform>(entity) else {
        return Some(point);
    };
    if transform.scale.x.abs() <= f32::EPSILON || transform.scale.y.abs() <= f32::EPSILON {
        return None;
    }

    let translated = point - transform.position;
    let sin = transform.rotation.sin();
    let cos = transform.rotation.cos();
    let rotated = Vector2::new(
        translated.x * cos + translated.y * sin,
        -translated.x * sin + translated.y * cos,
    );

    Some(rotated / transform.scale)
}

fn apply_transform(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    let Ok(transform) = world.get::<&Transform>(entity) else {
        return;
    };

    canvas.translate((transform.position.x, transform.position.y));
    canvas.rotate(transform.rotation.to_degrees(), None);
    canvas.scale((transform.scale.x, transform.scale.y));
}

fn transform_matrix(transform: &Transform) -> skia_safe::Matrix {
    affine_matrix(transform.position, transform.scale, transform.rotation)
}

fn camera_matrix(transform: &CameraTransform) -> skia_safe::Matrix {
    let inverse_zoom = if transform.zoom.abs() <= f32::EPSILON {
        0.0
    } else {
        transform.zoom.recip()
    };

    affine_matrix(
        transform.position,
        Vector2::splat(inverse_zoom),
        transform.rotation,
    )
}

fn affine_matrix(position: Vector2, scale: Vector2, rotation: f32) -> skia_safe::Matrix {
    let sin = rotation.sin();
    let cos = rotation.cos();

    skia_safe::Matrix::new_all(
        cos * scale.x,
        -sin * scale.y,
        position.x,
        sin * scale.x,
        cos * scale.y,
        position.y,
        0.0,
        0.0,
        1.0,
    )
}

fn draw_outline(bounds: skia_safe::Rect, canvas: &skia_safe::Canvas) {
    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(skia_safe::PaintStyle::Stroke);
    paint.set_color4f(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.8), None);
    paint.set_stroke_width(3.0);
    canvas.draw_rect(bounds, &paint);

    paint.set_color4f(skia_safe::Color4f::new(1.0, 1.0, 1.0, 0.9), None);
    paint.set_stroke_width(1.0);
    canvas.draw_rect(bounds, &paint);
}

fn union_bounds(left: skia_safe::Rect, right: skia_safe::Rect) -> skia_safe::Rect {
    skia_safe::Rect::new(
        left.left.min(right.left),
        left.top.min(right.top),
        left.right.max(right.right),
        left.bottom.max(right.bottom),
    )
}
