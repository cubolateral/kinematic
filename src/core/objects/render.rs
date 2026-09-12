use crate::core::{
    components::{Camera2D, Camera3D, Draw2D, Draw3D, Node, Simulation, Transform2D},
    objects::{
        CanvasSettings, CanvasTexture, GlobalTransform, ProjectionSource, bounds3d,
        draw_projection_2d, global_matrix3d, local_transform,
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
    let settings = world.get::<&CanvasSettings>(scope).ok()?;
    let camera = world.get::<&Camera3D>(scope).ok()?;
    camera.validate().ok()?;
    let ray = camera_ray(&camera, settings.resolution, point)?;

    crate::core::objects::child_iter(world, scope)
        .filter_map(|child| pick_entity3d(world, child, ray))
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, entity)| entity)
}

pub(crate) fn outline_segments3d(
    world: &hecs::World,
    scope: hecs::Entity,
    target: hecs::Entity,
) -> Option<Vec<[[f32; 2]; 2]>> {
    fn contains(world: &hecs::World, entity: hecs::Entity, target: hecs::Entity) -> bool {
        if world.get::<&CanvasSettings>(entity).is_ok()
            || !world
                .get::<&Node>(entity)
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
        .get::<&Node>(scope)
        .is_ok_and(|node| node.is_activated)
        || !world
            .get::<&Draw3D>(scope)
            .is_ok_and(|draw| draw.visibility)
        || !crate::core::objects::child_iter(world, scope)
            .any(|child| contains(world, child, target))
    {
        return None;
    }

    let settings = world.get::<&CanvasSettings>(scope).ok()?;
    let camera = world.get::<&Camera3D>(scope).ok()?;
    camera.validate().ok()?;
    let (min, max) = bounds3d(world, target)?;
    let transform = global_matrix3d(world, target);
    let view_projection =
        camera_projection(&camera, settings.resolution)? * camera.matrix().inverse();
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
    .map(|corner| project_point(view_projection, transform.transform_point3(corner)))
    .into_iter()
    .collect::<Option<Vec<_>>>()?;
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
            .get::<&Node>(entity)
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

fn project_point(view_projection: glam::Mat4, point: glam::Vec3) -> Option<[f32; 2]> {
    let clip = view_projection * point.extend(1.0);
    if !clip.is_finite() || clip.w <= f32::EPSILON {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    Some([ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5])
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
