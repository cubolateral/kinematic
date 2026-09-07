use crate::core::{
    components::{Draw, Node, Transform2D},
    objects::{CameraTransform2D, CanvasSettings, GlobalTransform, children, local_transform},
    types::Vector2,
};

pub(crate) fn active_camera_matrix(
    world: &hecs::World,
    root: hecs::Entity,
) -> Option<skia_safe::Matrix> {
    let mut camera = None;
    if world.get::<&CanvasSettings>(root).is_ok() {
        for child in children(world, root) {
            find_active_camera(world, child, GlobalTransform::default(), &mut camera);
        }
    } else {
        find_active_camera(world, root, GlobalTransform::default(), &mut camera);
    }
    camera
}

fn find_active_camera(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: GlobalTransform,
    camera: &mut Option<skia_safe::Matrix>,
) {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return;
    }
    let node = world
        .get::<&Node>(entity)
        .expect("Camera traversal object must contain a Node component.");
    if !node.is_activated {
        return;
    }

    let global = parent.append(local_transform(world, entity));
    let matrix = transform_matrix(global);

    if world.get::<&CameraTransform2D>(entity).is_ok() && matrix.invert().is_some() {
        *camera = Some(matrix);
    }

    for child in children(world, entity) {
        find_active_camera(world, child, global, camera);
    }
}

pub(crate) fn draw_entity(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    draw_entity_with_parent(world, entity, GlobalTransform::default(), canvas);
}

fn draw_entity_with_parent(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: GlobalTransform,
    canvas: &skia_safe::Canvas,
) {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return;
    }
    let node = world
        .get::<&Node>(entity)
        .expect("Drawn object must contain a Node component.");
    if !node.is_activated {
        return;
    }

    let Ok(draw) = world.get::<&Draw>(entity) else {
        return;
    };
    let opacity = draw.opacity.clamp(0.0, 1.0);
    if opacity <= 0.0 {
        return;
    }

    let children = node.children.clone().unwrap_or_default();
    let global = parent.append(local_transform(world, entity));
    let save_count = canvas.save();
    apply_global_transform(parent, global, canvas);

    if children.is_empty() || opacity >= 1.0 {
        (draw.on_draw)(world, entity, canvas, opacity);

        for child in children {
            draw_entity_with_parent(world, child, global, canvas);
        }
    } else {
        let layer_count = canvas.save_layer_alpha_f(None, opacity);
        (draw.on_draw)(world, entity, canvas, 1.0);

        for child in children {
            draw_entity_with_parent(world, child, global, canvas);
        }

        canvas.restore_to_count(layer_count);
    }

    canvas.restore_to_count(save_count);
}

pub(crate) fn draw_entity_outline(
    world: &hecs::World,
    entity: hecs::Entity,
    target: hecs::Entity,
    thickness: f32,
    canvas: &skia_safe::Canvas,
) -> bool {
    draw_entity_outline_with_parent(
        world,
        entity,
        target,
        GlobalTransform::default(),
        thickness,
        canvas,
    )
}

fn draw_entity_outline_with_parent(
    world: &hecs::World,
    entity: hecs::Entity,
    target: hecs::Entity,
    parent: GlobalTransform,
    thickness: f32,
    canvas: &skia_safe::Canvas,
) -> bool {
    if world.get::<&CanvasSettings>(entity).is_ok() || world.get::<&Draw>(entity).is_err() {
        return false;
    }
    let node = world
        .get::<&Node>(entity)
        .expect("Outlined object must contain a Node component.");
    if !node.is_activated {
        return false;
    }

    let global = parent.append(local_transform(world, entity));
    let save_count = canvas.save();
    apply_global_transform(parent, global, canvas);

    let found = if entity == target {
        if let Some(bounds) = local_bounds(world, entity) {
            draw_outline(bounds, thickness, canvas);
        }

        true
    } else {
        children(world, entity).into_iter().any(|child| {
            draw_entity_outline_with_parent(world, child, target, global, thickness, canvas)
        })
    };

    canvas.restore_to_count(save_count);
    found
}

pub(crate) fn pick_entity(
    world: &hecs::World,
    entity: hecs::Entity,
    point: Vector2,
) -> Option<hecs::Entity> {
    pick_entity_with_parent(world, entity, point, GlobalTransform::default())
}

fn pick_entity_with_parent(
    world: &hecs::World,
    entity: hecs::Entity,
    point: Vector2,
    parent: GlobalTransform,
) -> Option<hecs::Entity> {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return None;
    }
    let node = world
        .get::<&Node>(entity)
        .expect("Picked object must contain a Node component.");
    let draw = world.get::<&Draw>(entity).ok()?;

    if !node.is_activated || draw.opacity <= 0.0 {
        return None;
    }

    let global = parent.append(local_transform(world, entity));
    let local_point = inverse_transform_point(global, point)?;

    if let Some(child) = node
        .children
        .as_ref()
        .into_iter()
        .flatten()
        .rev()
        .copied()
        .find_map(|child| pick_entity_with_parent(world, child, point, global))
    {
        return Some(child);
    }

    let bounds = local_bounds(world, entity)?;

    (local_point.x >= bounds.left
        && local_point.x <= bounds.right
        && local_point.y >= bounds.top
        && local_point.y <= bounds.bottom)
        .then_some(entity)
}

#[doc(hidden)]
pub fn object_box(world: &hecs::World, entity: hecs::Entity) -> Vector2 {
    local_bounds(world, entity)
        .map(|bounds| Vector2::new(bounds.width(), bounds.height()))
        .unwrap_or(Vector2::ZERO)
}

fn local_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return None;
    }
    let draw = world.get::<&Draw>(entity).ok()?;
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
    let Ok(transform) = world.get::<&Transform2D>(entity) else {
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

fn inverse_transform_point(transform: GlobalTransform, point: Vector2) -> Option<Vector2> {
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

fn apply_global_transform(
    parent: GlobalTransform,
    global: GlobalTransform,
    canvas: &skia_safe::Canvas,
) {
    let Some(inverse_parent) = transform_matrix(parent).invert() else {
        return;
    };
    let relative = skia_safe::Matrix::concat(&inverse_parent, &transform_matrix(global));
    canvas.concat(&relative);
}

fn transform_matrix(transform: GlobalTransform) -> skia_safe::Matrix {
    affine_matrix(transform.position, transform.scale, transform.rotation)
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

fn draw_outline(bounds: skia_safe::Rect, thickness: f32, canvas: &skia_safe::Canvas) {
    let mut paint = skia_safe::Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(skia_safe::PaintStyle::Stroke);
    paint.set_color4f(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.8), None);
    paint.set_stroke_width(thickness * 3.0);
    canvas.draw_rect(bounds, &paint);

    paint.set_color4f(skia_safe::Color4f::new(1.0, 1.0, 1.0, 0.9), None);
    paint.set_stroke_width(thickness);
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

/// Draws one canvas scope using only its explicitly associated camera.
pub(crate) fn draw_canvas2d(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    let settings = world.get::<&CanvasSettings>(entity).unwrap();
    let [r, g, b, a] = settings.clear.rgba();
    canvas.clear(skia_safe::Color4f::new(r, g, b, a));
    let saved = canvas.save();
    canvas.translate((
        settings.resolution.0 as f32 * 0.5,
        settings.resolution.1 as f32 * 0.5,
    ));
    if let Some(camera) = settings
        .camera
        .and_then(|camera| camera_view_matrix(world, camera))
        .or_else(|| active_camera_matrix(world, entity))
    {
        if let Some(view) = camera.invert() {
            canvas.concat(&view);
        }
    }
    for child in children(world, entity) {
        draw_entity(world, child, canvas);
    }
    canvas.restore_to_count(saved);
}

pub(crate) fn draw_canvas_outline2d(
    world: &hecs::World,
    scope: hecs::Entity,
    target: hecs::Entity,
    thickness: f32,
    canvas: &skia_safe::Canvas,
) {
    let saved = canvas.save();
    if let Some(view) = canvas_camera_matrix(world, scope).and_then(|matrix| matrix.invert()) {
        canvas.concat(&view);
    }
    for child in children(world, scope) {
        draw_entity_outline(world, child, target, thickness, canvas);
    }
    canvas.restore_to_count(saved);
}

pub(crate) fn pick_canvas2d(
    world: &hecs::World,
    scope: hecs::Entity,
    point: Vector2,
) -> Option<hecs::Entity> {
    let point = canvas_camera_matrix(world, scope).map_or(point, |matrix| {
        let point = matrix.map_point((point.x, point.y));
        Vector2::new(point.x, point.y)
    });
    children(world, scope)
        .into_iter()
        .rev()
        .find_map(|child| pick_entity(world, child, point))
}

fn canvas_camera_matrix(world: &hecs::World, scope: hecs::Entity) -> Option<skia_safe::Matrix> {
    let settings = world.get::<&CanvasSettings>(scope).ok()?;
    settings
        .camera
        .and_then(|camera| camera_view_matrix(world, camera))
        .or_else(|| active_camera_matrix(world, scope))
}

fn camera_view_matrix(world: &hecs::World, camera: hecs::Entity) -> Option<skia_safe::Matrix> {
    world.get::<&CameraTransform2D>(camera).ok()?;
    Some(transform_matrix(crate::core::objects::global_transform(
        world, camera,
    )))
}
