use crate::core::{
    components::{Camera2D, Camera3D, Draw2D, Draw3D, Simulation, TreeNode},
    objects::{
        CanvasSettings, CanvasTexture, GlobalTransform, ProjectionSource, bounds3d,
        draw_projection_2d, global_matrix3d, global_transform, local_transform,
    },
    types::Vector2,
};
use skia_safe::QuickReject;
use std::collections::HashMap;

#[derive(Clone, Copy)]
enum AppearanceActivity {
    Evaluated,
    At { root: hecs::Entity, time: f32 },
}

#[derive(Clone, Copy)]
struct AppearanceMode {
    activity: AppearanceActivity,
    root_opacity: f32,
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
    camera_base: Option<&skia_safe::M44>,
    follows_camera: bool,
) {
    draw_entity_with_mode(
        world,
        entity,
        parent,
        canvas,
        images,
        None,
        camera_base,
        follows_camera,
    );
}

fn draw_entity_with_mode(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: GlobalTransform,
    canvas: &skia_safe::Canvas,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
    appearance: Option<AppearanceMode>,
    camera_base: Option<&skia_safe::M44>,
    follows_camera: bool,
) {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return;
    }
    let node = world
        .get::<&TreeNode>(entity)
        .expect("Drawn object must contain a Node component.");
    let active = match appearance.map(|mode| mode.activity) {
        Some(AppearanceActivity::At { root, .. }) if entity == root => true,
        Some(AppearanceActivity::At { time, .. }) => {
            node.lifetime[0] <= time && time < node.lifetime[1]
        }
        _ => node.is_activated,
    };
    if !active {
        return;
    }

    let Ok(draw) = world.get::<&Draw2D>(entity) else {
        return;
    };
    let opacity = appearance
        .filter(
            |mode| matches!(mode.activity, AppearanceActivity::At { root, .. } if root == entity),
        )
        .map_or(draw.opacity, |mode| mode.root_opacity)
        .clamp(0.0, 1.0);
    if !draw.visibility || opacity <= 0.0 {
        return;
    }

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

    if appearance.is_none() && draw_creation_appearance(world, entity, canvas, opacity) {
        canvas.restore_to_count(save_count);
        return;
    }

    let mixes_camera_spaces = follows_camera
        && camera_base.is_some()
        && opacity < 1.0
        && subtree_ignores_camera(world, entity);
    let bounds = if mixes_camera_spaces {
        None
    } else if children.len() == 0 || opacity < 1.0 {
        appearance
            .and_then(|mode| appearance_bounds(world, entity, mode.activity))
            .or_else(|| visual_bounds(world, entity, global, transform_matrix(global).invert()))
    } else {
        None
    };
    if bounds.is_some_and(|bounds| canvas.quick_reject(&bounds)) {
        canvas.restore_to_count(save_count);
        return;
    }

    let composites_opacity = children.len() != 0 || world.get::<&Simulation>(entity).is_ok();
    if !composites_opacity || opacity >= 1.0 {
        draw_object_appearance(world, entity, canvas, opacity, images, appearance.is_none());

        for child in children {
            draw_entity_with_mode(
                world,
                child,
                global,
                canvas,
                images,
                appearance,
                camera_base,
                follows_camera,
            );
        }
    } else {
        let layer_count = canvas.save_layer_alpha_f(bounds, opacity);
        draw_object_appearance(world, entity, canvas, 1.0, images, appearance.is_none());

        for child in children {
            draw_entity_with_mode(
                world,
                child,
                global,
                canvas,
                images,
                appearance,
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

fn draw_object_appearance(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
    images: Option<&HashMap<CanvasTexture, skia_safe::Image>>,
    present_effects: bool,
) {
    if present_effects {
        if world
            .get::<&crate::core::objects::TextShape>(entity)
            .is_ok()
            && crate::core::objects::text_2d::draw_text_effect(world, entity, canvas, opacity)
        {
            return;
        }
        if world
            .get::<&crate::core::objects::Latex2DShape>(entity)
            .is_ok()
            && crate::core::objects::draw_latex_effect(world, entity, canvas, opacity)
        {
            return;
        }
    }
    let draw = world.get::<&Draw2D>(entity).unwrap();
    (draw.on_draw)(world, entity, canvas, opacity);
    draw_projection_2d_entity(world, entity, canvas, opacity, images);
}

fn appearance_bounds(
    world: &hecs::World,
    entity: hecs::Entity,
    activity: AppearanceActivity,
) -> Option<skia_safe::Rect> {
    let draw = world.get::<&Draw2D>(entity).ok()?;
    let explicit_root = matches!(activity, AppearanceActivity::At { root, .. } if root == entity);
    if !draw.visibility || (!explicit_root && draw.opacity <= 0.0) {
        return None;
    }
    let own = (draw.visual_bounds)(world, entity);
    let mut bounds = (!own.is_empty()).then_some(own);
    for child in children_by_z_index(world, entity) {
        let node = world.get::<&TreeNode>(child).unwrap();
        let active = match activity {
            AppearanceActivity::Evaluated => node.is_activated,
            AppearanceActivity::At { time, .. } => {
                node.lifetime[0] <= time && time < node.lifetime[1]
            }
        };
        if !active {
            continue;
        }
        let Some(child_bounds) = appearance_bounds(world, child, activity) else {
            continue;
        };
        let transformed = transform_matrix(local_transform(world, child))
            .map_rect(child_bounds)
            .0;
        bounds = Some(bounds.map_or(transformed, |bounds| union_bounds(bounds, transformed)));
    }
    bounds
}

fn draw_creation_appearance(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    opacity: f32,
) -> bool {
    use crate::core::{components::Morph, objects::CreationDraw, types::Color};
    use std::hash::{Hash, Hasher};

    let morph = world.get::<&Morph>(entity).unwrap();
    if !morph.particles_enabled || morph.progress >= 1.0 {
        return false;
    }
    let Some(bounds) = appearance_bounds(world, entity, AppearanceActivity::Evaluated) else {
        return false;
    };
    let mut recorder = skia_safe::PictureRecorder::new();
    let target = recorder.begin_recording(bounds, false);
    if let Some(inverse) = transform_matrix(local_transform(world, entity)).invert() {
        target.concat(&inverse);
    }
    draw_entity_with_mode(
        world,
        entity,
        GlobalTransform::default(),
        target,
        None,
        Some(AppearanceMode {
            activity: AppearanceActivity::Evaluated,
            root_opacity: 1.0,
        }),
        None,
        false,
    );
    let Some(picture) = recorder.finish_recording_as_picture(None) else {
        return false;
    };
    let density = (2048.0 / bounds.width().max(bounds.height()).max(1.0)).min(2.0);
    let dimensions = (
        (bounds.width() * density).ceil().max(1.0) as i32,
        (bounds.height() * density).ceil().max(1.0) as i32,
    );
    let Some(mut surface) = skia_safe::surfaces::raster_n32_premul(dimensions) else {
        return false;
    };
    surface.canvas().clear(skia_safe::colors::TRANSPARENT);
    surface.canvas().scale((density, density));
    surface.canvas().translate((-bounds.left, -bounds.top));
    surface.canvas().draw_picture(&picture, None, None);
    let pixels = surface.peek_pixels().unwrap();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    pixels.bytes().unwrap().hash(&mut hasher);
    let visual_key = hasher.finish();
    let pixel_color = |point: Vector2| {
        let x = ((point.x - bounds.left) * density) as i32;
        let y = ((point.y - bounds.top) * density) as i32;
        if x < 0 || y < 0 || x >= dimensions.0 || y >= dimensions.1 {
            return Color::TRANSPARENT;
        }
        let color = pixels.get_color((x, y));
        Color::new(
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            color.a() as f32 / 255.0,
        )
    };
    let style = world
        .get::<&crate::core::components::Style>(entity)
        .map_or_else(
            |_| crate::core::components::Style::default(),
            |style| (*style).clone(),
        );
    CreationDraw {
        entity,
        cache_slot: 0,
        bounds,
        visual_key,
        style: &style,
        pixel_color: Some(&pixel_color),
        morph: &morph,
        opacity,
        canvas,
    }
    .render(|target, target_opacity| {
        let layer = target.save_layer_alpha_f(bounds, target_opacity);
        target.draw_picture(&picture, None, None);
        target.restore_to_count(layer);
    })
}

pub(crate) fn capture_appearance(
    world: &hecs::World,
    entity: hecs::Entity,
    parent: hecs::Entity,
    time: f32,
    opacity: f32,
) -> crate::core::objects::particle::Silhouette {
    let activity = AppearanceActivity::At { root: entity, time };
    let local_bounds = appearance_bounds(world, entity, activity)
        .unwrap_or_else(|| skia_safe::Rect::from_xywh(0.0, 0.0, 1.0, 1.0));
    let mut bounds = transform_matrix(local_transform(world, entity))
        .map_rect(local_bounds)
        .0;
    bounds.outset((2.0, 2.0));
    let mut recorder = skia_safe::PictureRecorder::new();
    let canvas = recorder.begin_recording(bounds, false);
    draw_entity_with_mode(
        world,
        entity,
        global_transform(world, parent),
        canvas,
        None,
        Some(AppearanceMode {
            activity,
            root_opacity: opacity,
        }),
        None,
        false,
    );
    let picture = recorder.finish_recording_as_picture(None).unwrap();
    crate::core::objects::particle::Silhouette::capture(bounds, |canvas| {
        canvas.draw_picture(&picture, None, None);
    })
}

fn visual_bounds(
    world: &hecs::World,
    entity: hecs::Entity,
    global: GlobalTransform,
    inverse: Option<skia_safe::Matrix>,
) -> Option<skia_safe::Rect> {
    use crate::core::components::Morph;
    use crate::core::objects::{Latex2DShape, TextShape, particle::ParticleTransform};
    let inverse = inverse?;
    if world
        .get::<&Morph>(entity)
        .is_ok_and(|m| m.particles_enabled)
        || world.get::<&ParticleTransform>(entity).is_ok()
        || world.get::<&TextShape>(entity).is_ok()
        || world.get::<&Latex2DShape>(entity).is_ok()
    {
        return None;
    }
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

fn logical_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    if world.get::<&CanvasSettings>(entity).is_ok() {
        return None;
    }
    let draw = world.get::<&Draw2D>(entity).ok()?;
    let size = (draw.box_size)(world, entity);
    let own = (size.x > 0.0 && size.y > 0.0)
        .then(|| skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y));
    let children = crate::core::objects::child_iter(world, entity)
        .filter(|child| {
            world
                .get::<&TreeNode>(*child)
                .is_ok_and(|node| node.is_activated)
        })
        .filter_map(|child| {
            let bounds = logical_bounds(world, child)?;
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

    match (own, child_bounds) {
        (Some(own), Some(children)) => Some(union_bounds(own, children)),
        (Some(own), None) => Some(own),
        (None, children) => children,
    }
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

pub(crate) fn draw_canvas2d_editor_with_images(
    world: &hecs::World,
    entity: hecs::Entity,
    canvas: &skia_safe::Canvas,
    images: &HashMap<CanvasTexture, skia_safe::Image>,
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
            Some(&camera_base),
            true,
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
    let width = resolution.0.max(1) as f32;
    let height = resolution.1.max(1) as f32;
    let tan = (camera.camera_fov * 0.5).tan();
    let local_direction = glam::vec3(
        point.x * 2.0 / width * width / height * tan,
        -point.y * 2.0 / height * tan,
        -1.0,
    )
    .normalize();
    let rotation = crate::core::normalized_quaternion(camera.camera_rotation);
    let direction = rotation * local_direction;
    direction.is_finite().then_some(Ray3D {
        origin: camera.camera_position,
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
    let projection = glam::camera::rh::proj::opengl::perspective(
        camera.camera_fov,
        aspect,
        camera.camera_near,
        camera.camera_far,
    );
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
