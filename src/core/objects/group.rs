use kinematic_macros::Object;

use crate::core::{
    components::{Draw, Node, Transform},
    objects::ObjectHandler,
    types::Vector2,
};

/// Scene object that draws an ordered tree of child objects.
#[derive(Object, hecs::Bundle)]
pub struct Group {
    #[trackable]
    pub transform: Transform,
    #[trackable]
    pub draw: Draw,
    pub children: Vec<hecs::Entity>,
}

impl Default for Group {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            draw: Draw {
                on_draw: draw_group,
                get_box: |world, entity| {
                    local_bounds(world, entity)
                        .map(|bounds| Vector2::new(bounds.width(), bounds.height()))
                        .unwrap_or(Vector2::ZERO)
                },
                ..Default::default()
            },
            children: vec![],
        }
    }
}

impl GroupHandler {
    /// Adds an object subtree to this group at the current scheduling time.
    pub fn add(&self, handler: &impl ObjectHandler) {
        let child = handler.get_id();
        let world = self.world.borrow();

        assert!(
            world.contains(child),
            "Added object must belong to this scene."
        );
        assert_ne!(
            self.entity, child,
            "A group must not be added as its own child."
        );
        assert!(
            !world
                .get::<&Node>(child)
                .expect("Added object must contain a Node component.")
                .is_root,
            "The scene root must not be added as a child."
        );
        assert!(
            !contains_entity(&world, child, self.entity),
            "Adding this object would create a group cycle."
        );

        for (group, children) in world.query::<(hecs::Entity, &Vec<hecs::Entity>)>().iter() {
            if !children.contains(&child) {
                continue;
            }

            assert_eq!(
                group, self.entity,
                "An object must not belong to more than one group."
            );
            return;
        }

        world
            .get::<&mut Vec<hecs::Entity>>(self.entity)
            .expect("Group handler must contain children.")
            .push(child);
        activate_subtree(&world, child, self.animator.time());
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

    if draw.opacity <= 0.0 {
        return;
    }

    let transform = world
        .get::<&Transform>(entity)
        .expect("Drawn object must contain a Transform component.");
    let save_count = canvas.save();

    canvas.translate((transform.position.x, transform.position.y));
    canvas.rotate(transform.rotation.to_degrees(), None);
    canvas.scale((transform.scale.x, transform.scale.y));

    (draw.on_draw)(world, entity, canvas);

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

    let transform = world
        .get::<&Transform>(entity)
        .expect("Outlined object must contain a Transform component.");
    let save_count = canvas.save();

    canvas.translate((transform.position.x, transform.position.y));
    canvas.rotate(transform.rotation.to_degrees(), None);
    canvas.scale((transform.scale.x, transform.scale.y));

    let found = if entity == target {
        if let Some(bounds) = local_bounds(world, entity) {
            draw_outline(bounds, canvas);
        }

        true
    } else {
        world
            .get::<&Vec<hecs::Entity>>(entity)
            .map(|children| {
                children
                    .iter()
                    .copied()
                    .any(|child| draw_entity_outline(world, child, target, canvas))
            })
            .unwrap_or(false)
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

    let transform = world
        .get::<&Transform>(entity)
        .expect("Picked object must contain a Transform component.");
    let point = inverse_transform_point(point, &transform)?;

    if let Ok(children) = world.get::<&Vec<hecs::Entity>>(entity) {
        if let Some(child) = children
            .iter()
            .rev()
            .copied()
            .find_map(|child| pick_entity(world, child, point))
        {
            return Some(child);
        }
    }

    let bounds = local_bounds(world, entity)?;

    (point.x >= bounds.left
        && point.x <= bounds.right
        && point.y >= bounds.top
        && point.y <= bounds.bottom)
        .then_some(entity)
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

fn inverse_transform_point(point: Vector2, transform: &Transform) -> Option<Vector2> {
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

fn local_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    if let Ok(children) = world.get::<&Vec<hecs::Entity>>(entity) {
        return children
            .iter()
            .copied()
            .filter(|child| {
                world
                    .get::<&Node>(*child)
                    .map(|node| node.is_activated)
                    .unwrap_or(false)
            })
            .filter_map(|child| transformed_bounds(world, child))
            .reduce(union_bounds);
    }

    let draw = world
        .get::<&Draw>(entity)
        .expect("Bounded object must contain a Draw component.");
    let size = (draw.get_box)(world, entity);

    (size.x > 0.0 && size.y > 0.0)
        .then(|| skia_safe::Rect::from_xywh(-size.x * 0.5, -size.y * 0.5, size.x, size.y))
}

fn transformed_bounds(world: &hecs::World, entity: hecs::Entity) -> Option<skia_safe::Rect> {
    let bounds = local_bounds(world, entity)?;
    let transform = world
        .get::<&Transform>(entity)
        .expect("Bounded object must contain a Transform component.");
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

fn union_bounds(left: skia_safe::Rect, right: skia_safe::Rect) -> skia_safe::Rect {
    skia_safe::Rect::new(
        left.left.min(right.left),
        left.top.min(right.top),
        left.right.max(right.right),
        left.bottom.max(right.bottom),
    )
}

fn draw_group(world: &hecs::World, entity: hecs::Entity, canvas: &skia_safe::Canvas) {
    let draw = world
        .get::<&Draw>(entity)
        .expect("Group must contain a Draw component.");
    let children = world
        .get::<&Vec<hecs::Entity>>(entity)
        .expect("Group must contain children.");
    let opacity = draw.opacity.clamp(0.0, 1.0);

    if opacity >= 1.0 {
        for child in children.iter().copied() {
            draw_entity(world, child, canvas);
        }
        return;
    }

    let save_count = canvas.save_layer_alpha_f(None, opacity);

    for child in children.iter().copied() {
        draw_entity(world, child, canvas);
    }

    canvas.restore_to_count(save_count);
}

fn contains_entity(world: &hecs::World, root: hecs::Entity, target: hecs::Entity) -> bool {
    if root == target {
        return true;
    }

    let Ok(children) = world.get::<&Vec<hecs::Entity>>(root) else {
        return false;
    };

    children
        .iter()
        .copied()
        .any(|child| contains_entity(world, child, target))
}

fn activate_subtree(world: &hecs::World, entity: hecs::Entity, time: f32) {
    world
        .get::<&mut Node>(entity)
        .expect("Added object must contain a Node component.")
        .activate(time);

    let children = world
        .get::<&Vec<hecs::Entity>>(entity)
        .map(|children| (*children).clone())
        .unwrap_or_default();

    for child in children {
        activate_subtree(world, child, time);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Scene, components::*, objects::*, types::*};

    use super::{local_bounds, pick_entity};

    #[test]
    fn handlers_expose_ids_and_groups_store_direct_children() {
        let mut scene = Scene::new();
        let circle = Circle::builder().build(&mut scene);
        let group = Group::builder().build(&mut scene);
        let root = scene.get_root();

        group.add(&circle);
        root.add(&group);

        let world = scene.get_world();
        let group_children = world.get::<&Vec<hecs::Entity>>(group.get_id()).unwrap();
        let root_children = world.get::<&Vec<hecs::Entity>>(root.get_id()).unwrap();

        assert_eq!(group_children.as_slice(), &[circle.get_id()]);
        assert_eq!(root_children.as_slice(), &[group.get_id()]);
    }

    #[test]
    fn adding_and_removing_a_group_updates_the_whole_subtree_lifetime() {
        let mut scene = Scene::new();
        let circle = Circle::builder().build(&mut scene);
        let group = Group::builder().build(&mut scene);
        let root = scene.get_root();

        group.add(&circle);
        root.add(&group);
        scene.wait(2.0);
        group.remove();

        {
            let world = scene.get_world();
            let group_node = world.get::<&Node>(group.get_id()).unwrap();
            let circle_node = world.get::<&Node>(circle.get_id()).unwrap();

            assert_eq!(group_node.lifetime, [0.0, 2.0]);
            assert_eq!(circle_node.lifetime, [0.0, 2.0]);
        }

        scene.update(1.0);
        {
            let world = scene.get_world();
            assert!(world.get::<&Node>(group.get_id()).unwrap().is_activated);
            assert!(world.get::<&Node>(circle.get_id()).unwrap().is_activated);
        }

        scene.update(2.0);
        let world = scene.get_world();

        assert!(!world.get::<&Node>(group.get_id()).unwrap().is_activated);
        assert!(!world.get::<&Node>(circle.get_id()).unwrap().is_activated);
    }

    #[test]
    fn removing_a_parent_preserves_an_earlier_child_removal() {
        let mut scene = Scene::new();
        let circle = Circle::builder().build(&mut scene);
        let group = Group::builder().build(&mut scene);
        let root = scene.get_root();

        group.add(&circle);
        root.add(&group);
        scene.wait(1.0);
        circle.remove();
        scene.wait(1.0);
        group.remove();

        let world = scene.get_world();
        let group_node = world.get::<&Node>(group.get_id()).unwrap();
        let circle_node = world.get::<&Node>(circle.get_id()).unwrap();

        assert_eq!(group_node.lifetime, [0.0, 2.0]);
        assert_eq!(circle_node.lifetime, [0.0, 1.0]);
    }

    #[test]
    #[should_panic(expected = "An object must not belong to more than one group.")]
    fn objects_reject_multiple_parents() {
        let mut scene = Scene::new();
        let circle = Circle::builder().build(&mut scene);
        let first = Group::builder().build(&mut scene);
        let second = Group::builder().build(&mut scene);

        first.add(&circle);
        second.add(&circle);
    }

    #[test]
    #[should_panic(expected = "Adding this object would create a group cycle.")]
    fn groups_reject_cycles() {
        let mut scene = Scene::new();
        let parent = Group::builder().build(&mut scene);
        let child = Group::builder().build(&mut scene);

        parent.add(&child);
        child.add(&parent);
    }

    #[test]
    fn group_opacity_is_applied_once_to_the_composed_children() {
        let mut scene = Scene::new();
        let first = Rect::builder()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .build(&mut scene);
        let second = Rect::builder()
            .size(vec2(8.0, 8.0))
            .fill(Color::RED)
            .build(&mut scene);
        let group = Group::builder().opacity(0.5).build(&mut scene);

        group.add(&first);
        group.add(&second);
        scene.get_root().add(&group);

        let image_info = skia_safe::ImageInfo::new(
            (16, 16),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Premul,
            None,
        );
        let mut surface = skia_safe::surfaces::raster(&image_info, None, None).unwrap();
        let canvas = surface.canvas();
        canvas.clear(skia_safe::colors::TRANSPARENT);
        canvas.translate((8.0, 8.0));

        scene.draw(canvas);

        let pixels = surface.peek_pixels().unwrap();
        let center = pixels.get_color((8, 8));

        assert_eq!(center.r(), 255);
        assert!((127..=128).contains(&center.a()));

        let world = scene.get_world();
        assert_eq!(world.get::<&Draw>(group.get_id()).unwrap().opacity, 0.5);
    }

    #[test]
    fn group_bounds_include_child_position_scale_and_rotation() {
        let mut scene = Scene::new();
        let rectangle = Rect::builder()
            .size(vec2(10.0, 20.0))
            .position(vec2(30.0, -10.0))
            .scale(vec2(2.0, 1.0))
            .rotation(std::f32::consts::FRAC_PI_2)
            .build(&mut scene);
        let group = Group::builder().build(&mut scene);

        group.add(&rectangle);

        let world = scene.get_world();
        let bounds = local_bounds(&world, group.get_id()).unwrap();

        assert!((bounds.left - 20.0).abs() < 0.001);
        assert!((bounds.top + 20.0).abs() < 0.001);
        assert!((bounds.right - 40.0).abs() < 0.001);
        assert!(bounds.bottom.abs() < 0.001);
    }

    #[test]
    fn picking_returns_the_frontmost_visible_object() {
        let mut scene = Scene::new();
        let behind = Rect::builder().size(vec2(20.0, 20.0)).build(&mut scene);
        let front = Rect::builder().size(vec2(20.0, 20.0)).build(&mut scene);
        let root = scene.get_root();

        root.add(&behind);
        root.add(&front);

        {
            let world = scene.get_world();

            assert_eq!(
                pick_entity(&world, root.get_id(), Vector2::ZERO),
                Some(front.get_id())
            );
        }

        front.opacity(0.0);
        let world = scene.get_world();

        assert_eq!(
            pick_entity(&world, root.get_id(), Vector2::ZERO),
            Some(behind.get_id())
        );
    }

    #[test]
    fn picking_empty_space_inside_a_group_selects_the_group() {
        let mut scene = Scene::new();
        let left = Rect::builder()
            .size(vec2(10.0, 10.0))
            .position(vec2(-20.0, 0.0))
            .build(&mut scene);
        let right = Rect::builder()
            .size(vec2(10.0, 10.0))
            .position(vec2(20.0, 0.0))
            .build(&mut scene);
        let group = Group::builder().build(&mut scene);
        let root = scene.get_root();

        group.add(&left);
        group.add(&right);
        root.add(&group);

        let world = scene.get_world();

        assert_eq!(
            pick_entity(&world, root.get_id(), Vector2::ZERO),
            Some(group.get_id())
        );
        assert_eq!(pick_entity(&world, root.get_id(), vec2(100.0, 100.0)), None);
    }
}
