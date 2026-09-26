use kinematic::prelude::*;

#[test]
fn two_dimensional_anchors_align_objects() {
    let mut scene = Scene::new();
    let other = rect()
        .size(vec2(20.0, 10.0))
        .position(vec2(50.0, 30.0))
        .build(&mut scene);
    let object = rect()
        .size(vec2(8.0, 4.0))
        .position(vec2(10.0, 20.0))
        .build(&mut scene);

    assert_eq!(other.middle(), vec2(50.0, 30.0));
    assert_eq!(other.left(), vec2(40.0, 30.0));
    assert_eq!(other.right(), vec2(60.0, 30.0));
    assert_eq!(other.top(), vec2(50.0, 25.0));
    assert_eq!(other.bottom(), vec2(50.0, 35.0));
    assert_eq!(other.top_left(), vec2(40.0, 25.0));
    assert_eq!(other.top_right(), vec2(60.0, 25.0));
    assert_eq!(other.bottom_left(), vec2(40.0, 35.0));
    assert_eq!(other.bottom_right(), vec2(60.0, 35.0));

    assert_eq!(object.left_to(other.right()), vec2(64.0, 30.0));
    assert_eq!(object.top_to(other.bottom()), vec2(50.0, 37.0));
    object.position(object.left_to(other.right())).immediate();
    assert_eq!(object.left(), other.right());
}

#[test]
fn three_dimensional_anchors_align_objects() {
    let mut scene = Scene::new();
    let other = prism()
        .size(vec3(20.0, 10.0, 6.0))
        .position(vec3(50.0, 30.0, 12.0))
        .build(&mut scene);
    let object = prism().size(vec3(8.0, 4.0, 2.0)).build(&mut scene);

    assert_eq!(other.middle(), vec3(50.0, 30.0, 12.0));
    assert_eq!(other.top(), vec3(50.0, 35.0, 12.0));
    assert_eq!(other.bottom(), vec3(50.0, 25.0, 12.0));
    assert_eq!(other.front(), vec3(50.0, 30.0, 15.0));
    assert_eq!(other.back(), vec3(50.0, 30.0, 9.0));
    assert_eq!(other.top_left_front(), vec3(40.0, 35.0, 15.0));
    assert_eq!(other.bottom_right_back(), vec3(60.0, 25.0, 9.0));

    assert_eq!(object.back_to(other.front()), vec3(50.0, 30.0, 16.0));
    assert_eq!(
        object.top_left_front_to(other.bottom_right_back()),
        vec3(64.0, 23.0, 8.0)
    );
    object.position(object.back_to(other.front())).immediate();
    assert_eq!(object.back(), other.front());
}

#[test]
fn group_anchor_uses_offset_child_bounds() {
    let mut scene = Scene::new();
    let margin = rect()
        .size(vec2(20.0, 20.0))
        .position(vec2(0.0, 100.0))
        .build(&mut scene);
    let group = group_2d().build(&mut scene);
    let child = rect()
        .size(vec2(10.0, 10.0))
        .position(vec2(0.0, 20.0))
        .build(&mut scene);
    group.add(&child);

    assert_eq!(group.box_size(), vec2(10.0, 10.0));
    assert_eq!(group.bottom_to(margin.bottom()), vec2(0.0, 85.0));
    group.position(group.bottom_to(margin.bottom())).immediate();
    assert_eq!(group.bottom(), margin.bottom());
}

#[test]
fn three_dimensional_group_anchor_uses_offset_child_bounds() {
    let mut scene = Scene::new();
    let group = group_3d().build(&mut scene);
    let child = prism()
        .size(vec3(10.0, 10.0, 10.0))
        .position(vec3(0.0, 0.0, 20.0))
        .build(&mut scene);
    group.add(&child);

    assert_eq!(group.box_size(), vec3(10.0, 10.0, 25.0));
    assert_eq!(group.front_to(vec3(0.0, 0.0, 100.0)), vec3(0.0, 0.0, 75.0));
    group
        .position(group.front_to(vec3(0.0, 0.0, 100.0)))
        .immediate();
    assert_eq!(group.front(), vec3(0.0, 0.0, 100.0));
}

#[test]
fn anchors_include_children_scheduled_for_the_current_time() {
    let mut scene = Scene::new();
    scene.wait(1.0);

    let group_2d = group_2d().build(&mut scene);
    let child_2d = rect()
        .size(vec2(10.0, 10.0))
        .position(vec2(0.0, 20.0))
        .build(&mut scene);
    group_2d.add(&child_2d);

    let group_3d = group_3d().build(&mut scene);
    let child_3d = prism()
        .size(vec3(10.0, 10.0, 10.0))
        .position(vec3(0.0, 0.0, 20.0))
        .build(&mut scene);
    group_3d.add(&child_3d);

    assert_eq!(group_2d.box_size(), Vector2::ZERO);
    assert_eq!(group_3d.box_size(), Vector3::ZERO);
    assert_eq!(group_2d.bottom_to(vec2(0.0, 100.0)), vec2(0.0, 75.0));
    assert_eq!(
        group_3d.front_to(vec3(0.0, 0.0, 100.0)),
        vec3(0.0, 0.0, 75.0)
    );
}

#[test]
fn future_nested_group_bottom_aligns_after_play() {
    let mut scene = Scene::new();
    let margin = rect()
        .size(vec2(20.0, 20.0))
        .position(vec2(0.0, 100.0))
        .build(&mut scene);
    scene.world_2d().add(&margin);
    scene.wait(1.0);

    let rule_group = group_2d().follows_camera(false).build(&mut scene);
    scene.world_2d().add(&rule_group);
    let patterns_group = group_2d().position_y(64.0).build(&mut scene);
    rule_group.add(&patterns_group);
    let pattern = group_2d().build(&mut scene);
    patterns_group.add(&pattern);
    let child = rect()
        .size(vec2(10.0, 10.0))
        .position_y(20.0)
        .build(&mut scene);
    pattern.add(&child);

    scene.all(|_| {
        rule_group
            .position(rule_group.bottom_to(margin.bottom()))
            .play();
    });
    scene.update(2.0);

    assert_eq!(rule_group.bottom(), margin.bottom());
}
