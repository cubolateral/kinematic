use crate::core::{
    components::{Camera2D, Draw2D, Node, Simulation, Transform2D},
    objects::{
        CanvasSettings, CanvasTexture, GlobalTransform, ProjectionSource, draw_projection_2d,
        local_transform,
    },
    types::Vector2,
};
use skia_safe::QuickReject;
use std::collections::HashMap;

pub(crate) fn draw_entity(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    draw_entity_with_parent(world, entity, GlobalTransform::default(), canvas, None);
}

fn draw_entity_with_parent(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: GlobalTransform,
    canvas: &skia_safe::Canvas,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
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

    let Ok(draw) = world.get::<&Draw2D>(entity) else {
        return;
    };
    let opacity = draw.opacity.clamp(0.0, 1.0);
    if !draw.visibility || opacity <= 0.0 {
        return;
    }

    let children = children_by_z_index(world, entity);
    let global = parent.append(local_transform(world, entity));
    let save_count = canvas.save();
    apply_global_transform(parent, global, canvas);

    let bounds = if children.len() == 0 || opacity < 1.0 {
        visual_bounds(world, entity, global, transform_matrix(global).invert())
    } else {
        None
    };
    if bounds.is_some_and(|bounds| canvas.quick_reject(&bounds)) {
        canvas.restore_to_count(save_count);
        return;
    }

    let composites_opacity = children.len() != 0 || world.get::<&Simulation>(entity).is_ok();
    if !composites_opacity || opacity >= 1.0 {
        (draw.on_draw)(world, entity, canvas, opacity);
        draw_projection_2d_entity(world, entity, canvas, opacity, images);

        for child in children {
            draw_entity_with_parent(world, child, global, canvas, images);
        }
    } else {
        let layer_count = canvas.save_layer_alpha_f(bounds, opacity);
        (draw.on_draw)(world, entity, canvas, 1.0);
        draw_projection_2d_entity(world, entity, canvas, 1.0, images);

        for child in children {
            draw_entity_with_parent(world, child, global, canvas, images);
        }

        canvas.restore_to_count(layer_count);
    }

    canvas.restore_to_count(save_count);
}

// Effect and text bounds can extend beyond get_box; retain Skia's clip in those cases.
fn visual_bounds(
    world: &hecs::World,
    entity: hecs::Entity,
    global: GlobalTransform,
    inverse: Option<skia_safe::Matrix>,
) -> Option<skia_safe::Rect> {
    use crate::core::components::{Morph, Style, stroke_width_for_scale};
    use crate::core::objects::{LatexShape, TextShape, particle::ParticleTransform};
    let inverse = inverse?;
    if world
        .get::<&Morph>(entity)
        .is_ok_and(|m| m.particles_enabled)
        || world.get::<&ParticleTransform>(entity).is_ok()
        || world.get::<&TextShape>(entity).is_ok()
        || world.get::<&LatexShape>(entity).is_ok()
    {
        return None;
    }
    let draw = world.get::<&Draw2D>(entity).ok()?;
    let size = (draw.get_box)(world, entity);
    let mut bounds = if size.x > 0.0 && size.y > 0.0 {
        let padding = world.get::<&Style>(entity).map_or(0.0, |style| {
            let scale = world
                .get::<&Transform2D>(entity)
                .map_or(Vector2::ONE, |t| t.scale);
            stroke_width_for_scale(style.stroke_width.max(0.0), scale) * 2.0
        });
        let local = skia_safe::Rect::from_xywh(
            -size.x * 0.5 - padding,
            -size.y * 0.5 - padding,
            size.x + padding * 2.0,
            size.y + padding * 2.0,
        );
        Some(
            skia_safe::Matrix::concat(&inverse, &transform_matrix(global))
                .map_rect(local)
                .0,
        )
    } else {
        None
    };
    for child in crate::core::objects::child_iter(world, entity) {
        if world.get::<&CanvasSettings>(child).is_ok()
            || !world.get::<&Node>(child).is_ok_and(|n| n.is_activated)
            || !world
                .get::<&Draw2D>(child)
                .is_ok_and(|d| d.visibility && d.opacity > 0.0)
        {
            continue;
        }
        let child_bounds = visual_bounds(
            world,
            child,
            global.append(local_transform(world, child)),
            Some(inverse),
        )?;
        bounds = Some(bounds.map_or(child_bounds, |bounds| union_bounds(bounds, child_bounds)));
    }
    bounds
}

fn draw_projection_2d_entity(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
) {
    let Ok(source) = world.get::<&ProjectionSource>(entity) else {
        return;
    };
    let Some(image) = source
        .0
        .and_then(|source| images.and_then(|images| images.get(&source)))
    else {
        return;
    };
    draw_projection_2d(world, entity, image, canvas, opacity);
}

pub(crate) fn outline_points(
    world: &hecs::World,
    scope: hecs::Entity,
    target: hecs::Entity,
) -> Option<[skia_safe::Point; 4]> {
    fn visit(
        world: &hecs::World,
        entity: hecs::Entity,
        target: hecs::Entity,
        parent: GlobalTransform,
    ) -> Option<[skia_safe::Point; 4]> {
        if world.get::<&CanvasSettings>(entity).is_ok()
            || !world
                .get::<&Node>(entity)
                .is_ok_and(|node| node.is_activated)
            || !world
                .get::<&Draw2D>(entity)
                .is_ok_and(|draw| draw.visibility)
        {
            return None;
        }
        let global = parent.append(local_transform(world, entity));
        if entity == target {
            let bounds = local_bounds(world, entity)?;
            let mut points = [
                skia_safe::Point::new(bounds.left, bounds.top),
                skia_safe::Point::new(bounds.right, bounds.top),
                skia_safe::Point::new(bounds.right, bounds.bottom),
                skia_safe::Point::new(bounds.left, bounds.bottom),
            ];
            transform_matrix(global).map_points_inplace(&mut points);
            return Some(points);
        }
        crate::core::objects::child_iter(world, entity)
            .find_map(|child| visit(world, child, target, global))
    }
    if !world
        .get::<&Node>(scope)
        .is_ok_and(|node| node.is_activated)
        || !world
            .get::<&Draw2D>(scope)
            .is_ok_and(|draw| draw.visibility)
    {
        return None;
    }
    let mut points = crate::core::objects::child_iter(world, scope)
        .find_map(|child| visit(world, child, target, GlobalTransform::default()))?;
    if let Some(view) = camera_matrix2d(world, scope).and_then(|m| m.invert()) {
        view.map_points_inplace(&mut points);
    }
    Some(points)
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
    let draw = world.get::<&Draw2D>(entity).ok()?;

    if !node.is_activated || !draw.visibility || draw.opacity <= 0.0 {
        return None;
    }

    let global = parent.append(local_transform(world, entity));
    let local_point = inverse_transform_point(global, point)?;

    if let Some(child) = children_by_z_index(world, entity)
        .into_iter()
        .rev()
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

pub(crate) fn children_by_z_index(
    world: &hecs::World,
    entity: hecs::Entity,
) -> impl DoubleEndedIterator<Item = hecs::Entity> + ExactSizeIterator + '_ {
    let node = world
        .get::<&Node>(entity)
        .expect("Scene object must contain a Node component.");
    let children = node.children.as_deref().unwrap_or_default();
    let z_index = |entity| world.get::<&Draw2D>(entity).map_or(0, |draw| draw.z_index);
    let sorted = if children
        .windows(2)
        .any(|pair| z_index(pair[0]) > z_index(pair[1]))
    {
        let mut sorted = children.to_vec();
        sorted.sort_by_key(|entity| z_index(*entity));
        Some(sorted)
    } else {
        None
    };
    let count = children.len();
    (0..count).map(move |index| {
        sorted
            .as_deref()
            .unwrap_or_else(|| node.children.as_deref().unwrap_or_default())[index]
    })
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
    let draw = world.get::<&Draw2D>(entity).ok()?;
    let size = (draw.get_box)(world, entity);
    let own = (size.x > 0.0 && size.y > 0.0)
        .then(|| skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y));
    let child_bounds = crate::core::objects::child_iter(world, entity)
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

fn union_bounds(left: skia_safe::Rect, right: skia_safe::Rect) -> skia_safe::Rect {
    skia_safe::Rect::new(
        left.left.min(right.left),
        left.top.min(right.top),
        left.right.max(right.right),
        left.bottom.max(right.bottom),
    )
}

/// Draws one canvas scope using only its explicitly associated camera.
#[cfg(test)]
pub(crate) fn draw_canvas2d(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    draw_canvas2d_inner(world, entity, canvas, None);
}

pub(crate) fn draw_canvas2d_with_images(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    images: &HashMap<CanvasTexture, skia_safe::Image>,
) {
    draw_canvas2d_inner(world, entity, canvas, Some(images));
}

fn draw_canvas2d_inner(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
) {
    let settings = world.get::<&CanvasSettings>(entity).unwrap();
    let [r, g, b, a] = settings.clear.rgba();
    canvas.clear(skia_safe::Color4f::new(r, g, b, a));
    if !world
        .get::<&Draw2D>(entity)
        .is_ok_and(|draw| draw.visibility)
    {
        return;
    }
    let saved = canvas.save();
    canvas.translate((
        settings.resolution.0 as f32 * 0.5,
        settings.resolution.1 as f32 * 0.5,
    ));
    if let Some(camera) = camera_matrix2d(world, entity) {
        if let Some(view) = camera.invert() {
            canvas.concat(&view);
        }
    }
    for child in children_by_z_index(world, entity) {
        draw_entity_with_parent(world, child, GlobalTransform::default(), canvas, images);
    }
    canvas.restore_to_count(saved);
}

pub(crate) fn pick_canvas2d(
    world: &hecs::World,
    scope: hecs::Entity,
    point: Vector2,
) -> Option<hecs::Entity> {
    if !world
        .get::<&Draw2D>(scope)
        .is_ok_and(|draw| draw.visibility)
    {
        return None;
    }
    let point = canvas_camera_matrix(world, scope).map_or(point, |matrix| {
        let point = matrix.map_point((point.x, point.y));
        Vector2::new(point.x, point.y)
    });
    children_by_z_index(world, scope)
        .into_iter()
        .rev()
        .find_map(|child| pick_entity(world, child, point))
}

fn canvas_camera_matrix(world: &hecs::World, scope: hecs::Entity) -> Option<skia_safe::Matrix> {
    world.get::<&CanvasSettings>(scope).ok()?;
    camera_matrix2d(world, scope)
}

pub(crate) fn camera_matrix2d(
    world: &hecs::World,
    canvas: hecs::Entity,
) -> Option<skia_safe::Matrix> {
    let camera = world.get::<&Camera2D>(canvas).ok()?;
    Some(transform_matrix(camera.transform()))
}
