use crate::core::{
    components::{Camera2D, Camera3D, Camera3DMode, Draw2D, Draw3D, Filter, Simulation, TreeNode},
    objects::{
        CanvasSettings, CanvasTexture, GlobalTransform, ProjectionSource, bounds3d,
        draw_projection_2d, global_matrix3d, local_transform,
    },
    types::Vector2,
};
use std::collections::HashMap;

pub(crate) struct ImageShaderImage {
    pub(crate) image: skia_safe::Image,
    pub(crate) bounds: skia_safe::Rect,
}

pub(crate) fn draw_entity(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    camera_base: Option<&skia_safe::M44>,
) {
    draw_entity_with_parent(
        world,
        entity,
        GlobalTransform::default(),
        canvas,
        None,
        None,
        None,
        camera_base,
        camera_base.is_some(),
    );
}

fn draw_entity_with_parent(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: GlobalTransform,
    canvas: &skia_safe::Canvas,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
    shader_images: Option<&HashMap<hecs::Entity, ImageShaderImage>>,
    skip_shader: Option<hecs::Entity>,
    camera_base: Option<&skia_safe::M44>,
    follows_camera: bool,
) {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return;
    }
    let node = world
        .get::<&TreeNode>(entity)
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
    let image_filter = world
        .get::<&Filter>(entity)
        .ok()
        .and_then(|filter| image_filter(&filter));
    let filtered = image_filter.is_some();

    let children = children_by_z_index(world, entity);
    let global = parent.append(local_transform(world, entity));
    let save_count = canvas.save();
    let follows_camera = follows_camera && draw.follows_camera;
    if !follows_camera && let Some(camera_base) = camera_base {
        canvas.set_matrix(camera_base);
        apply_global_transform(GlobalTransform::default(), global, canvas);
    } else {
        apply_global_transform(parent, global, canvas);
    }

    if skip_shader != Some(entity)
        && let Some(shader_image) = shader_images.and_then(|images| images.get(&entity))
    {
        canvas.draw_image_rect(
            &shader_image.image,
            None,
            shader_image.bounds,
            &skia_safe::Paint::default(),
        );
        canvas.restore_to_count(save_count);
        return;
    }

    let mixes_camera_spaces = follows_camera
        && camera_base.is_some()
        && (opacity < 1.0 || filtered)
        && subtree_ignores_camera(world, entity);
    let bounds = if mixes_camera_spaces {
        None
    } else if children.len() == 0 || opacity < 1.0 || filtered {
        visual_bounds(world, entity, global, transform_matrix(global).invert())
    } else {
        None
    };
    let composites = children.len() != 0 || world.get::<&Simulation>(entity).is_ok();
    if !filtered && (!composites || opacity >= 1.0) {
        draw_object(world, entity, canvas, opacity, images);

        for child in children {
            draw_entity_with_parent(
                world,
                child,
                global,
                canvas,
                images,
                shader_images,
                skip_shader,
                camera_base,
                follows_camera,
            );
        }
    } else {
        let mut paint = skia_safe::Paint::default();
        paint.set_alpha_f(opacity);
        paint.set_image_filter(image_filter);
        let mut layer = skia_safe::canvas::SaveLayerRec::default().paint(&paint);
        if let Some(bounds) = bounds.as_ref() {
            layer = layer.bounds(bounds);
        }
        let layer_count = canvas.save_layer(&layer);
        draw_object(world, entity, canvas, 1.0, images);

        for child in children {
            draw_entity_with_parent(
                world,
                child,
                global,
                canvas,
                images,
                shader_images,
                skip_shader,
                camera_base,
                follows_camera,
            );
        }

        canvas.restore_to_count(layer_count);
    }

    canvas.restore_to_count(save_count);
}

fn subtree_ignores_camera(world: &hecs::World, entity: hecs::Entity) -> bool {
    crate::core::objects::child_iter(world, entity).any(|child| {
        world
            .get::<&Draw2D>(child)
            .is_ok_and(|draw| !draw.follows_camera)
            || subtree_ignores_camera(world, child)
    })
}

fn draw_object(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
) {
    let draw = world.get::<&Draw2D>(entity).unwrap();
    (draw.on_draw)(world, entity, canvas, opacity);
    draw_projection_2d_entity(world, entity, canvas, opacity, images);
}

fn subtree_visual_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    let draw = world.get::<&Draw2D>(entity).ok()?;
    if !draw.visibility || draw.opacity <= 0.0 {
        return None;
    }
    let own = (draw.visual_bounds)(world, entity);
    let mut bounds = (!own.is_empty()).then_some(own);
    for child in children_by_z_index(world, entity) {
        let node = world.get::<&TreeNode>(child).unwrap();
        if !node.is_activated {
            continue;
        }
        let Some(child_bounds) = subtree_visual_bounds(world, child) else {
            continue;
        };
        let transformed = transform_matrix(local_transform(world, child))
            .map_rect(child_bounds)
            .0;
        bounds = Some(bounds.map_or(transformed, |bounds| union_bounds(bounds, transformed)));
    }
    bounds.map(|bounds| filtered_bounds(world, entity, bounds))
}

fn filtered_bounds(
    world: &hecs::World,
    entity: hecs::Entity,
    bounds: skia_safe::Rect,
) -> skia_safe::Rect {
    world
        .get::<&Filter>(entity)
        .ok()
        .and_then(|filter| image_filter(&filter))
        .map_or(bounds, |filter| filter.compute_fast_bounds(bounds))
}

fn image_filter(filter: &Filter) -> Option<skia_safe::ImageFilter> {
    let blur = filter.blur.max(0.0);
    let mut image_filter = (blur > 0.0)
        .then(|| skia_safe::image_filters::blur((blur, blur), None, None, None))
        .flatten();
    if filter.shadow_color.a > 0.0 {
        let color = filter.shadow_color;
        image_filter = skia_safe::image_filters::drop_shadow(
            (filter.shadow_offset.x, filter.shadow_offset.y),
            (filter.shadow_blur.max(0.0), filter.shadow_blur.max(0.0)),
            skia_safe::Color4f::new(color.r, color.g, color.b, color.a),
            None,
            image_filter,
            None,
        );
    }
    image_filter
}

fn visual_bounds(
    world: &hecs::World,
    entity: hecs::Entity,
    global: GlobalTransform,
    inverse: Option<skia_safe::Matrix>,
) -> Option<skia_safe::Rect> {
    let inverse = inverse?;
    let draw = world.get::<&Draw2D>(entity).ok()?;
    let local = (draw.visual_bounds)(world, entity);
    let mut bounds = if !local.is_empty() {
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
            || !world.get::<&TreeNode>(child).is_ok_and(|n| n.is_activated)
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

#[cfg(test)]
pub(crate) fn outline_points(
    world: &hecs::World,
    scope: hecs::Entity,
    target: hecs::Entity,
) -> Option<[skia_safe::Point; 4]> {
    let mut points = outline_points_in_world(world, scope, target)?;
    if object_follows_camera(world, scope, target)
        && let Some(view) = camera_matrix2d(world, scope).and_then(|m| m.invert())
    {
        view.map_points_inplace(&mut points);
    }
    Some(points)
}

pub(crate) fn object_follows_camera(
    world: &hecs::World,
    scope: hecs::Entity,
    mut entity: hecs::Entity,
) -> bool {
    while entity != scope {
        if !world
            .get::<&Draw2D>(entity)
            .is_ok_and(|draw| draw.follows_camera)
        {
            return false;
        }
        let Some(parent) = world
            .get::<&TreeNode>(entity)
            .ok()
            .and_then(|node| node.parent)
        else {
            return false;
        };
        entity = parent;
    }
    true
}

pub(crate) fn outline_points_in_world(
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
                .get::<&TreeNode>(entity)
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
        .get::<&TreeNode>(scope)
        .is_ok_and(|node| node.is_activated)
        || !world
            .get::<&Draw2D>(scope)
            .is_ok_and(|draw| draw.visibility)
    {
        return None;
    }
    crate::core::objects::child_iter(world, scope)
        .find_map(|child| visit(world, child, target, GlobalTransform::default()))
}

fn pick_entity_with_parent(
    world: &hecs::World,
    entity: hecs::Entity,
    camera_point: Vector2,
    fixed_point: Vector2,
    parent: GlobalTransform,
    follows_camera: bool,
) -> Option<hecs::Entity> {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return None;
    }
    let node = world
        .get::<&TreeNode>(entity)
        .expect("Picked object must contain a Node component.");
    let draw = world.get::<&Draw2D>(entity).ok()?;

    if !node.is_activated || !draw.visibility || draw.opacity <= 0.0 {
        return None;
    }

    let follows_camera = follows_camera && draw.follows_camera;
    let point = if follows_camera {
        camera_point
    } else {
        fixed_point
    };
    let global = parent.append(local_transform(world, entity));
    let local_point = inverse_transform_point(global, point)?;

    if let Some(child) = children_by_z_index(world, entity)
        .into_iter()
        .rev()
        .find_map(|child| {
            pick_entity_with_parent(
                world,
                child,
                camera_point,
                fixed_point,
                global,
                follows_camera,
            )
        })
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
        .get::<&TreeNode>(entity)
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
    logical_bounds(world, entity)
        .map(|bounds| Vector2::new(bounds.width(), bounds.height()))
        .unwrap_or(Vector2::ZERO)
}

pub(crate) fn logical_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    logical_bounds_inner(world, entity, None)
}

pub(crate) fn logical_bounds_at(
    world: &hecs::World,
    entity: hecs::Entity,
    time: f32,
) -> Option<skia_safe::Rect> {
    logical_bounds_inner(world, entity, Some(time))
}

fn logical_bounds_inner(
    world: &hecs::World,
    entity: hecs::Entity,
    time: Option<f32>,
) -> Option<skia_safe::Rect> {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return None;
    }
    let draw = world.get::<&Draw2D>(entity).ok()?;
    let size = (draw.box_size)(world, entity);
    let bounds = draw.logical_bounds.map_or_else(
        || skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y),
        |bounds| bounds(world, entity),
    );
    let own = (!bounds.is_empty()).then_some(bounds);
    let children = crate::core::objects::child_iter(world, entity)
        .filter(|child| {
            world.get::<&TreeNode>(*child).is_ok_and(|node| {
                time.map_or(node.is_activated, |time| {
                    node.lifetime[0] <= time && time < node.lifetime[1]
                })
            })
        })
        .filter_map(|child| {
            let bounds = logical_bounds_inner(world, child, time)?;
            Some(
                transform_matrix(local_transform(world, child))
                    .map_rect(bounds)
                    .0,
            )
        })
        .reduce(union_bounds);
    match (own, children) {
        (Some(own), Some(children)) => Some(union_bounds(own, children)),
        (Some(own), None) => Some(own),
        (None, children) => children,
    }
}

fn local_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return None;
    }
    let draw = world.get::<&Draw2D>(entity).ok()?;
    let bounds = (draw.visual_bounds)(world, entity);
    let own = (!bounds.is_empty()).then_some(bounds);
    let child_bounds = crate::core::objects::child_iter(world, entity)
        .filter(|child| {
            world
                .get::<&TreeNode>(*child)
                .map(|node| node.is_activated)
                .unwrap_or(false)
        })
        .filter_map(|child| transformed_bounds(world, child))
        .reduce(union_bounds);

    let bounds = match (own, child_bounds) {
        (Some(own), Some(children)) => Some(union_bounds(own, children)),
        (Some(own), None) => Some(own),
        (None, children) => children,
    };
    bounds.map(|bounds| filtered_bounds(world, entity, bounds))
}

fn transformed_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    let bounds = local_bounds(world, entity)?;
    Some(
        transform_matrix(local_transform(world, entity))
            .map_rect(bounds)
            .0,
    )
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
    draw_canvas2d_inner(world, entity, canvas, None, None);
}

pub(crate) fn draw_canvas2d_with_shader_images(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    images: &HashMap<CanvasTexture, skia_safe::Image>,
    shader_images: &HashMap<hecs::Entity, ImageShaderImage>,
) {
    draw_canvas2d_inner(world, entity, canvas, Some(images), Some(shader_images));
}

pub(crate) fn draw_canvas2d_editor_with_images(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    images: &HashMap<CanvasTexture, skia_safe::Image>,
    shader_images: &HashMap<hecs::Entity, ImageShaderImage>,
    target_size: (u32, u32),
    pan: [f32; 2],
    zoom: f32,
    correction: [f32; 2],
    camera_view: bool,
) {
    canvas.clear(skia_safe::colors::TRANSPARENT);
    if !world
        .get::<&Draw2D>(entity)
        .is_ok_and(|draw| draw.visibility)
    {
        return;
    }

    let saved = canvas.save();
    canvas.translate((target_size.0 as f32 * 0.5, target_size.1 as f32 * 0.5));
    let mut camera_base = None;
    if camera_view {
        canvas.scale((correction[0], correction[1]));
        camera_base = Some(canvas.local_to_device());
        if let Some(view) = camera_matrix2d(world, entity).and_then(|camera| camera.invert()) {
            canvas.concat(&view);
        }
    } else {
        canvas.translate((pan[0] * correction[0], pan[1] * correction[1]));
        canvas.scale((zoom * correction[0], zoom * correction[1]));
    }
    for child in children_by_z_index(world, entity) {
        draw_entity_with_parent(
            world,
            child,
            GlobalTransform::default(),
            canvas,
            Some(images),
            Some(shader_images),
            None,
            camera_base.as_ref(),
            camera_base.is_some(),
        );
    }
    if !camera_view && let Some(points) = camera_outline_points2d(world, entity, false) {
        let mask =
            skia_safe::Path::polygon(&points, true, skia_safe::PathFillType::InverseEvenOdd, None);
        let mut paint = skia_safe::Paint::new(skia_safe::Color4f::new(0.0, 0.0, 0.0, 0.25), None);
        paint.set_anti_alias(true);
        canvas.draw_path(&mask, &paint);
    }
    canvas.restore_to_count(saved);
}

fn draw_canvas2d_inner(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
    shader_images: Option<&HashMap<hecs::Entity, ImageShaderImage>>,
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
    let camera_base = canvas.local_to_device();
    if let Some(camera) = camera_matrix2d(world, entity) {
        if let Some(view) = camera.invert() {
            canvas.concat(&view);
        }
    }
    for child in children_by_z_index(world, entity) {
        draw_entity_with_parent(
            world,
            child,
            GlobalTransform::default(),
            canvas,
            images,
            shader_images,
            None,
            Some(&camera_base),
            true,
        );
    }
    canvas.restore_to_count(saved);
}

pub(crate) fn image_shader_bounds(
    world: &hecs::World,
    entity: hecs::Entity,
    padding: f32,
) -> Option<skia_safe::Rect> {
    let mut bounds = subtree_visual_bounds(world, entity)?;
    bounds.outset((padding, padding));
    Some(bounds)
}

pub(crate) fn draw_image_shader_source(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    images: &HashMap<CanvasTexture, skia_safe::Image>,
    shader_images: &HashMap<hecs::Entity, ImageShaderImage>,
) {
    let saved = canvas.save();
    if let Some(inverse) = transform_matrix(local_transform(world, entity)).invert() {
        canvas.concat(&inverse);
        draw_entity_with_parent(
            world,
            entity,
            GlobalTransform::default(),
            canvas,
            Some(images),
            Some(shader_images),
            Some(entity),
            None,
            false,
        );
    }
    canvas.restore_to_count(saved);
}

#[cfg(test)]
pub(crate) fn pick_canvas2d(
    world: &hecs::World,
    scope: hecs::Entity,
    point: Vector2,
) -> Option<hecs::Entity> {
    pick_canvas2d_with_view(world, scope, point, true)
}

fn pick_canvas2d_with_view(
    world: &hecs::World,
    scope: hecs::Entity,
    point: Vector2,
    camera_view: bool,
) -> Option<hecs::Entity> {
    if !world
        .get::<&Draw2D>(scope)
        .is_ok_and(|draw| draw.visibility)
    {
        return None;
    }
    let camera_point = camera_view
        .then(|| camera_matrix2d(world, scope))
        .flatten()
        .map_or(point, |matrix| {
            let point = matrix.map_point((point.x, point.y));
            Vector2::new(point.x, point.y)
        });
    children_by_z_index(world, scope)
        .into_iter()
        .rev()
        .find_map(|child| {
            pick_entity_with_parent(
                world,
                child,
                camera_point,
                point,
                GlobalTransform::default(),
                camera_view,
            )
        })
}

pub(crate) fn pick_canvas2d_in_world(
    world: &hecs::World,
    scope: hecs::Entity,
    point: Vector2,
    camera_view: bool,
) -> Option<hecs::Entity> {
    pick_canvas2d_with_view(world, scope, point, camera_view)
}

#[cfg(test)]
pub(crate) fn pick_canvas3d(
    world: &hecs::World,
    scope: hecs::Entity,
    point: Vector2,
) -> Option<hecs::Entity> {
    if !world
        .get::<&Draw3D>(scope)
        .is_ok_and(|draw| draw.visibility)
    {
        return None;
    }
    let camera = world.get::<&Camera3D>(scope).ok()?;
    let settings = world.get::<&CanvasSettings>(scope).ok()?;
    pick_canvas3d_with_camera(world, scope, &camera, settings.resolution, point)
}

pub(crate) fn pick_canvas3d_with_camera(
    world: &hecs::World,
    scope: hecs::Entity,
    camera: &Camera3D,
    resolution: (u32, u32),
    point: Vector2,
) -> Option<hecs::Entity> {
    if !world
        .get::<&Draw3D>(scope)
        .is_ok_and(|draw| draw.visibility)
    {
        return None;
    }
    camera.validate().ok()?;
    let ray = camera_ray(camera, resolution, point)?;

    crate::core::objects::child_iter(world, scope)
        .filter_map(|child| pick_entity3d(world, child, ray))
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, entity)| entity)
}

#[cfg(test)]
pub(crate) fn outline_segments3d(
    world: &hecs::World,
    scope: hecs::Entity,
    target: hecs::Entity,
) -> Option<Vec<[[f32; 2]; 2]>> {
    let settings = world.get::<&CanvasSettings>(scope).ok()?;
    let camera = world.get::<&Camera3D>(scope).ok()?;
    outline_segments3d_with_camera(world, scope, target, &camera, settings.resolution)
}

#[cfg(test)]
pub(crate) fn outline_segments3d_with_camera(
    world: &hecs::World,
    scope: hecs::Entity,
    target: hecs::Entity,
    camera: &Camera3D,
    resolution: (u32, u32),
) -> Option<Vec<[[f32; 2]; 2]>> {
    camera.validate().ok()?;
    let view_projection = camera_projection(camera, resolution)? * camera.matrix().inverse();
    outline_segments3d_in_world(world, scope, target)?
        .into_iter()
        .map(|[from, to]| {
            Some([
                project_point(view_projection, from)?,
                project_point(view_projection, to)?,
            ])
        })
        .collect()
}

pub(crate) fn outline_segments3d_in_world(
    world: &hecs::World,
    scope: hecs::Entity,
    target: hecs::Entity,
) -> Option<Vec<[glam::Vec3; 2]>> {
    fn contains(world: &hecs::World, entity: hecs::Entity, target: hecs::Entity) -> bool {
        if world.get::<&CanvasSettings>(entity).is_ok()
            || !world
                .get::<&TreeNode>(entity)
                .is_ok_and(|node| node.is_activated)
            || !world
                .get::<&Draw3D>(entity)
                .is_ok_and(|draw| draw.visibility)
        {
            return false;
        }
        entity == target
            || crate::core::objects::child_iter(world, entity)
                .any(|child| contains(world, child, target))
    }

    if !world
        .get::<&TreeNode>(scope)
        .is_ok_and(|node| node.is_activated)
        || !world
            .get::<&Draw3D>(scope)
            .is_ok_and(|draw| draw.visibility)
        || !crate::core::objects::child_iter(world, scope)
            .any(|child| contains(world, child, target))
    {
        return None;
    }

    let (min, max) = bounds3d(world, target)?;
    let transform = global_matrix3d(world, target);
    let corners = [
        glam::vec3(min.x, min.y, min.z),
        glam::vec3(max.x, min.y, min.z),
        glam::vec3(max.x, max.y, min.z),
        glam::vec3(min.x, max.y, min.z),
        glam::vec3(min.x, min.y, max.z),
        glam::vec3(max.x, min.y, max.z),
        glam::vec3(max.x, max.y, max.z),
        glam::vec3(min.x, max.y, max.z),
    ]
    .map(|corner| transform.transform_point3(corner));
    const EDGES: [(usize, usize); 12] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    Some(
        EDGES
            .into_iter()
            .map(|(from, to)| [corners[from], corners[to]])
            .collect(),
    )
}

#[derive(Clone, Copy)]
struct Ray3D {
    origin: glam::Vec3,
    direction: glam::Vec3,
    near: f32,
    far: f32,
}

fn camera_ray(camera: &Camera3D, resolution: (u32, u32), point: Vector2) -> Option<Ray3D> {
    let height = resolution.1.max(1) as f32;
    let rotation = crate::core::normalized_quaternion(camera.camera_rotation);
    let (origin, direction) = match camera.camera_mode {
        Camera3DMode::Perspective => {
            let tan = (camera.camera_fov * 0.5).tan();
            let direction = glam::vec3(
                point.x * 2.0 / height * tan,
                -point.y * 2.0 / height * tan,
                -1.0,
            )
            .normalize();
            (camera.camera_position, rotation * direction)
        }
        Camera3DMode::Orthogonal => {
            let offset = glam::vec3(
                point.x / height * camera.camera_fov,
                -point.y / height * camera.camera_fov,
                0.0,
            );
            (
                camera.camera_position + rotation * offset,
                rotation * -glam::Vec3::Z,
            )
        }
    };
    direction.is_finite().then_some(Ray3D {
        origin,
        direction,
        near: camera.camera_near,
        far: camera.camera_far,
    })
}

fn pick_entity3d(
    world: &hecs::World,
    entity: hecs::Entity,
    ray: Ray3D,
) -> Option<(f32, hecs::Entity)> {
    if world.get::<&CanvasSettings>(entity).is_ok()
        || !world
            .get::<&TreeNode>(entity)
            .is_ok_and(|node| node.is_activated)
        || !world
            .get::<&Draw3D>(entity)
            .is_ok_and(|draw| draw.visibility)
    {
        return None;
    }

    if let Some(hit) = crate::core::objects::child_iter(world, entity)
        .filter_map(|child| pick_entity3d(world, child, ray))
        .min_by(|left, right| left.0.total_cmp(&right.0))
    {
        return Some(hit);
    }

    let (min, max) = bounds3d(world, entity)?;
    let inverse = global_matrix3d(world, entity).inverse();
    if !inverse.is_finite() {
        return None;
    }
    let origin = inverse.transform_point3(ray.origin);
    let direction = inverse.transform_vector3(ray.direction);
    ray_cuboid_intersection(origin, direction, min, max, ray.near, ray.far)
        .map(|distance| (distance, entity))
}

fn ray_cuboid_intersection(
    origin: glam::Vec3,
    direction: glam::Vec3,
    min: glam::Vec3,
    max: glam::Vec3,
    mut near: f32,
    mut far: f32,
) -> Option<f32> {
    for axis in 0..3 {
        let origin = origin[axis];
        let direction = direction[axis];
        if direction.abs() <= f32::EPSILON {
            if origin < min[axis] || origin > max[axis] {
                return None;
            }
            continue;
        }
        let first = (min[axis] - origin) / direction;
        let second = (max[axis] - origin) / direction;
        near = near.max(first.min(second));
        far = far.min(first.max(second));
        if near > far {
            return None;
        }
    }
    Some(near)
}

#[cfg(test)]
fn camera_projection(camera: &Camera3D, resolution: (u32, u32)) -> Option<glam::Mat4> {
    let aspect = resolution.0 as f32 / resolution.1.max(1) as f32;
    let projection = match camera.camera_mode {
        Camera3DMode::Perspective => glam::camera::rh::proj::opengl::perspective(
            camera.camera_fov,
            aspect,
            camera.camera_near,
            camera.camera_far,
        ),
        Camera3DMode::Orthogonal => {
            let half_height = camera.camera_fov * 0.5;
            glam::camera::rh::proj::opengl::orthographic(
                -half_height * aspect,
                half_height * aspect,
                -half_height,
                half_height,
                camera.camera_near,
                camera.camera_far,
            )
        }
    };
    projection.is_finite().then_some(projection)
}

#[cfg(test)]
fn project_point(view_projection: glam::Mat4, point: glam::Vec3) -> Option<[f32; 2]> {
    let clip = view_projection * point.extend(1.0);
    if !clip.is_finite() || clip.w <= f32::EPSILON {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    Some([ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5])
}

pub(crate) fn camera_matrix2d(
    world: &hecs::World,
    canvas: hecs::Entity,
) -> Option<skia_safe::Matrix> {
    let camera = world.get::<&Camera2D>(canvas).ok()?;
    Some(transform_matrix(camera.transform()))
}

pub(crate) fn camera_outline_points2d(
    world: &hecs::World,
    canvas: hecs::Entity,
    camera_view: bool,
) -> Option<[skia_safe::Point; 4]> {
    let settings = world.get::<&CanvasSettings>(canvas).ok()?;
    let camera = camera_matrix2d(world, canvas)?;
    let half_width = settings.resolution.0 as f32 * 0.5;
    let half_height = settings.resolution.1 as f32 * 0.5;
    let mut points = [
        skia_safe::Point::new(-half_width, -half_height),
        skia_safe::Point::new(half_width, -half_height),
        skia_safe::Point::new(half_width, half_height),
        skia_safe::Point::new(-half_width, half_height),
    ];
    camera.map_points_inplace(&mut points);
    if camera_view {
        camera.invert()?.map_points_inplace(&mut points);
    }
    Some(points)
}

#[cfg(test)]
mod shader_capture_tests {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn group_blur_filters_its_subtree() {
        let mut scene = Scene::new_with_resolution((64, 64));
        let group = group_2d().blur(4.0).build(&mut scene);
        let child = rect()
            .size(vec2(16.0, 16.0))
            .fill(Color::WHITE)
            .build(&mut scene);
        group.add(&child);
        scene.world_2d().add(&group);
        scene.update(0.0);

        let mut surface = skia_safe::surfaces::raster_n32_premul((64, 64)).unwrap();
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        surface.canvas().translate((32.0, 32.0));
        draw_entity(&scene.world(), group.entity(), surface.canvas(), None);

        let pixels = surface.peek_pixels().unwrap();
        assert!(pixels.get_color((44, 32)).a() > 0);
    }

    #[test]
    fn transparent_shadow_skips_the_image_filter() {
        let filter = Filter {
            shadow_offset: vec2(8.0, 8.0),
            shadow_blur: 4.0,
            ..Default::default()
        };

        assert!(image_filter(&filter).is_none());
        assert!(
            image_filter(&Filter {
                shadow_color: Color::BLACK,
                ..filter
            })
            .is_some()
        );
    }
}
