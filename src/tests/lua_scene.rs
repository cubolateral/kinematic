use crate::{
    core::{
        ProjectContext,
        components::Transform2D,
        objects::{CircleShape, ObjectHandler},
    },
    scripting::{LuaScene, Sources},
};
use std::path::PathBuf;

fn sources(text: &str) -> Sources {
    [(PathBuf::from("intro.lua"), text.to_owned())].into()
}
fn load(text: &str) -> LuaScene {
    LuaScene::load(
        ProjectContext::new(".").unwrap(),
        "intro.lua",
        sources(text),
        (640, 480),
        60,
        (0, 0),
    )
    .unwrap()
}
const SCENE: &str = r##"
return function(s)
    local ball = s:circle {id='ball', radius=24, fill=k.color('#60A5FA'), position=k.vec2(-100, 0)}
    s:world_2d():add(ball)
    ball:to {position = k.vec2(100, 0)}:play(2, k.ease.linear)
    s:wait(1)
end
"##;

#[test]
fn lua_uses_native_tracks_and_seek() {
    let scene = load(SCENE);
    let entity = scene.entity("ball").unwrap();

    scene.update(1.0).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&Transform2D>(entity)
            .unwrap()
            .position
            .x,
        0.0
    );

    scene.update(0.0).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&Transform2D>(entity)
            .unwrap()
            .position
            .x,
        -100.0
    );

    assert_eq!(scene.duration(), 3.0);
}

#[test]
fn reload_preserves_ids_and_rejects_invalid_candidates() {
    let mut scene = load(SCENE);
    let entity = scene.entity("ball").unwrap();

    scene.update(1.0).unwrap();

    scene
        .reload(sources(&SCENE.replace("radius=24", "radius=48")))
        .unwrap();

    assert_eq!(scene.entity("ball"), Some(entity));

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(entity)
            .unwrap()
            .radius,
        48.0
    );

    assert!(scene.reload(sources("Invalid Lua ???")).is_err());
    assert_eq!(scene.entity("ball"), Some(entity));

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(entity)
            .unwrap()
            .radius,
        48.0
    );

    assert_eq!(scene.revision(), 2);
}

#[test]
fn signal_errors_do_not_commit_partial_writes() {
    let source=SCENE.replace("s:wait(1)", "ball:signal(function(obj, frame) obj.radius:set(99); if frame.time > 2 then error('Failed') end end); s:wait(1)");
    let scene = load(&source);

    assert!(scene.update(2.5).is_err());

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(scene.entity("ball").unwrap())
            .unwrap()
            .radius,
        24.0
    );
}

#[test]
fn camera_following_shader_rect_updates_without_signal_errors() {
    let scene = load(
        r#"
local shader = k.image_shader {source = [[#version 330 core
uniform vec2 center;
uniform float zoom;
uniform vec2 viewport;
in vec2 k_uv;
out vec4 k_color;
void main() { k_color = vec4(center.x, zoom / viewport.x, k_uv.x, 1.0); }
]]}
return function(s)
    local world = s:world_2d()
    local fractal = s:rect {
        id = 'mandelbrot', size = k.vec2(640, 480), shader = shader,
        uniforms = {center = k.vec2(-0.5, 0), zoom = 1, viewport = k.vec2(640, 480)},
    }
    world:add(fractal)
    fractal:signal(function(obj)
        local position = world.camera_position:get()
        local zoom = world.camera_zoom:get()
        obj.position:set(position)
        obj.rotation:set(world.camera_rotation:get())
        obj.scale:set(k.vec2(1 / zoom, 1 / zoom))
        obj:uniform('center'):set(k.vec2(-0.5 + position.x / 200, -position.y / 200))
        obj:uniform('zoom'):set(zoom)
    end)
    s:all(function()
        world.camera_position:to(k.vec2(100, 50)):play(1, k.ease.linear)
        world.camera_zoom:to(2):play(1, k.ease.linear)
        world.camera_rotation:to(0.4):play(1, k.ease.linear)
    end)
end
"#,
    );

    scene.update(0.5).unwrap();

    let entity = scene.entity("mandelbrot").unwrap();
    let world = scene.scene().world();
    let transform = world.get::<&Transform2D>(entity).unwrap();
    let camera_zoom = world
        .get::<&crate::core::components::Camera2D>(scene.scene().world_2d().entity())
        .unwrap()
        .camera_zoom;

    assert_eq!(transform.position.x, 50.0);
    assert!((transform.rotation - 0.2).abs() < 0.0001);
    assert!(
        (transform.scale.x - 2.0 / 3.0).abs() < 0.0001,
        "Scale: {}, camera zoom: {}",
        transform.scale.x,
        camera_zoom
    );
    assert_eq!(
        crate::core::objects::shader_uniform(&world, entity, "center"),
        Ok(crate::core::TrackValue::Vector2(crate::core::types::vec2(
            -0.25, -0.125
        )))
    );
}

#[test]
fn duplicate_ids_and_infinite_loops_are_diagnostics() {
    assert!(
        LuaScene::load(
            ProjectContext::new(".").unwrap(),
            "intro.lua",
            sources(&SCENE.replace("s:wait(1)", "s:circle {id='ball'}")),
            (640, 480),
            60,
            (0, 0)
        )
        .is_err()
    );

    assert!(
        LuaScene::load(
            ProjectContext::new(".").unwrap(),
            "intro.lua",
            sources("while true do end"),
            (640, 480),
            60,
            (0, 0)
        )
        .is_err()
    );
}

#[test]
fn hierarchy_anchors_and_manual_animation_share_native_behavior() {
    let scene = load(
        r#"
return function(s)
    local group = s:group_2d {id='group', position=k.vec2(50, 0)}
    local a = s:circle {id='a', position=k.vec2(10, 20), radius=2}
    local b = s:circle {id='b'}
    s:world_2d():add(group)
    group:add(a)
    group:insert(b, 1)
    assert(group:get_child(2):type_name() == 'Circle')
    assert(a:ancestor('Group2D'):type_name() == 'Group2D')
    assert(a:left().x == 8 and a:top().y == 18)
    assert(a:left_to(k.vec2(0, 0)).x == 2)
    assert(a:global_position().x == 60)
    local cube = s:cube {id='cube', position=k.vec3(3, 4, 5), size=k.vec3(2, 2, 2)}
    s:world_3d():add(cube)
    assert(cube:top_front().y == 5 and cube:top_front().z == 6)
    a.opacity:set(0)
    a.scale:set(k.vec2(0, 0))
    s:all(function()
        a:to {opacity = 1}:play(1, k.ease.linear)
        a:to {scale = k.vec2(1, 1)}:play(1, k.ease.linear)
    end)
    assert(not pcall(function() return a.fade_in end))
    assert(not pcall(function() return a.grow_in end))
    s:wait(1)
end
"#,
    );

    scene.update(0.5).unwrap();

    let world = scene.scene().world();
    let a = scene.entity("a").unwrap();

    assert_eq!(world.get::<&Transform2D>(a).unwrap().scale.x, 0.5);

    assert_eq!(
        world
            .get::<&crate::core::components::Draw2D>(a)
            .unwrap()
            .opacity,
        0.5
    );
}

#[test]
fn removing_a_shader_on_reload_removes_its_components() {
    use crate::core::objects::MeshShaderData;

    let source = r#"
return function(s)
    local shape = s:sphere {id='shape', shader=k.mesh_shader {fragment='out vec4 outColor; void main() { outColor=vec4(1.0); }'}}
    s:world_3d():add(shape)
    s:wait(1)
end
"#;
    let mut scene = load(source);
    let entity = scene.entity("shape").unwrap();

    assert!(scene.scene().world().get::<&MeshShaderData>(entity).is_ok());

    scene.reload(sources(&source.replace(
        ", shader=k.mesh_shader {fragment='out vec4 outColor; void main() { outColor=vec4(1.0); }'}", ""
    ))).unwrap();

    assert_eq!(scene.entity("shape"), Some(entity));

    assert!(
        scene
            .scene()
            .world()
            .get::<&MeshShaderData>(entity)
            .is_err()
    );

    scene.update(0.5).unwrap();
}

#[test]
fn a_failed_frame_restores_previous_tracks_and_successful_signals() {
    let scene = load(
        r#"
return function(s)
    local a = s:circle {id='a', radius=10}
    local b = s:circle {id='b', radius=20}
    s:world_2d():add(a)
    s:world_2d():add(b)
    a:signal(function(obj, frame)
        obj.radius:set(10 + frame.time)
        b.radius:set(20 + frame.time)
        if frame.time > 1 then b.opacity:set(0) end
    end)
    a:signal(function(_, frame) if frame.time > 1 then error('Failed frame.') end end)
    a:to {position = k.vec2(100, 0)}:play(2, k.ease.linear)
end
"#,
    );
    let a = scene.entity("a").unwrap();

    scene.update(0.5).unwrap();
    assert!(scene.update(1.5).is_err());

    let b = scene.entity("b").unwrap();

    assert_eq!(
        scene.scene().world().get::<&CircleShape>(b).unwrap().radius,
        20.5
    );

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&crate::core::components::Draw2D>(b)
            .unwrap()
            .opacity,
        1.0
    );

    assert_eq!(
        scene.scene().world().get::<&CircleShape>(a).unwrap().radius,
        10.5
    );

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&Transform2D>(a)
            .unwrap()
            .position
            .x,
        25.0
    );

    scene.update(0.0).unwrap();

    assert_eq!(
        scene.scene().world().get::<&CircleShape>(a).unwrap().radius,
        10.0
    );
}

#[test]
fn quaternion_tweens_keep_full_turns() {
    let scene = load(
        r#"
return function(s)
    local shape=s:sphere {id='spin'}
    s:world_3d():add(shape)
    shape.rotation:rotate(k.vec3(0,0,1), 2*math.pi):play(1, k.ease.linear)
end
"#,
    );

    scene.update(0.25).unwrap();

    let world = scene.scene().world();
    let transform = world
        .get::<&crate::core::components::Transform3D>(scene.entity("spin").unwrap())
        .unwrap();

    assert!((transform.rotation * glam::Vec3::X - glam::Vec3::Y).length() < 0.001);
}

fn try_load(text: &str) -> mlua::Result<LuaScene> {
    LuaScene::load(
        ProjectContext::new(".").unwrap(),
        "intro.lua",
        sources(text),
        (640, 480),
        60,
        (0, 0),
    )
}

#[test]
fn partial_channels_pairs_and_reusable_definitions_share_native_tracks() {
    let scene = load(
        r#"
return function(s)
    local ball = s:circle {id='ball',radius=16,position={x=-200},scale={y=2},fill={a=0.5}}
    s:world_2d():add(ball)
    assert(ball:get('position').y == 0 and ball:get('scale').x == 1)
    assert(ball:get('fill').a == 0.5)
    local movement = ball:by {position={x=100}}
    assert(ball:get('position.x') == -200)
    movement:play(1,k.ease.linear)
    movement:play(1,k.ease.linear)
    s:all(function()
        ball:from {position={x={0,200}},radius={16,39},opacity={0,1}}:play(2,k.ease.linear)
        ball:to {position={y=50}}:play(2,k.ease.linear)
    end)
end
"#,
    );
    let entity = scene.entity("ball").unwrap();

    scene.update(1.5).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&Transform2D>(entity)
            .unwrap()
            .position,
        glam::vec2(-50.0, 0.0)
    );

    scene.update(3.0).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&Transform2D>(entity)
            .unwrap()
            .position,
        glam::vec2(100.0, 25.0)
    );

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(entity)
            .unwrap()
            .radius,
        27.5
    );

    scene.update(0.0).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&Transform2D>(entity)
            .unwrap()
            .position,
        glam::vec2(-200.0, 0.0)
    );

    assert_eq!(scene.duration(), 4.0);
}

#[test]
fn omitted_channels_do_not_create_tracks_and_to_keeps_its_absolute_destination() {
    let scene = load(
        r#"
return function(s)
    local ball=s:circle{id='ball',position={y=12}}
    s:world_2d():add(ball)
    local destination=ball:to{position={x=10},radius=24}
    destination:play(1,k.ease.linear)
    ball:by{position={x=20}}:play(1)
    destination:play(1,k.ease.linear)
    ball:to{radius=30}:play()
end
"#,
    );
    let entity = scene.entity("ball").unwrap();

    scene.update(3.0).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&Transform2D>(entity)
            .unwrap()
            .position,
        glam::vec2(10.0, 12.0)
    );

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(entity)
            .unwrap()
            .radius,
        30.0
    );

    let world = scene.scene().world();
    let animation = world
        .get::<&crate::core::components::Animation>(entity)
        .unwrap();

    assert!(
        !animation
            .tracks
            .iter()
            .any(|t| t.track.target.name() == "position" && t.track.target.channel() == Some(1))
    );
}

#[test]
fn animation_definitions_only_expose_play() {
    load(
        r#"
return function(s)
    local ball=s:circle{}
    local label=s:text_2d{}
    local phase=s:track(0)
    local animations={
        ball:to{radius=24}, ball:by{radius=2}, ball:from{radius={16,24}},
        ball.radius:to(24), ball.radius:by(2), ball.radius:from({16,24}),
        phase:to(1), label:morph('New'), label:fade('New'), ball:reset(),
    }
    for _,animation in ipairs(animations) do
        assert(type(animation.play)=='function')
        for _,name in ipairs({'to','by','from','morph','fade'}) do
            assert(not pcall(function() animation[name](animation,{}) end))
        end
    end
end
"#,
    );
}

#[test]
fn malformed_pairs_channels_overlaps_and_limits_are_rejected() {
    for declaration in [
        "ball:from{radius={1}}",
        "ball:from{radius={1,2,3}}",
        "ball:from{position={k.vec2(0,0)}}",
        "ball:from{position={x={0,1,2}}}",
        "ball:from{radius={false,2}}",
        "ball:to{position={r=2}}",
        "ball:to{fill={x=0.2}}",
        "ball:by{opacity=2}:play(1)",
        "ball:to{radius=-1}",
        "ball:to{opacity=2}",
        "ball:to{position=k.vec2(1,2),['position.x']=3}",
        "ball:by{rotation=k.quat(0,0,0,1)}",
    ] {
        let source = format!("return function(s) local ball=s:circle{{}}; {declaration} end");

        assert!(
            try_load(&source).is_err(),
            "Accepted invalid declaration: {declaration}"
        );
    }

    for fields in [
        "position=k.vec2(1,2),['position.x']=3",
        "position={x=1},['position.x']=2",
        "fill={a=false}",
        "position={z=1}",
    ] {
        assert!(try_load(&format!("return function(s) s:circle{{{fields}}} end")).is_err());
    }

    assert!(try_load("return function(s) end, 'Extra'").is_err());
    assert!(try_load("return {build=function(s) end,name='Old'}").is_err());
}

#[test]
fn text_and_latex_fade_crossfade_both_strings_and_support_seeking() {
    for (constructor, old, new) in [
        ("text_2d", "I        ", "        I"),
        ("latex_2d", r"x\qquad\qquad", r"\qquad\qquad x"),
    ] {
        for (name, easing) in [
            ("linear", crate::core::Easing::Linear),
            ("cubic_in_out", crate::core::Easing::InOutCubic),
            ("quint_out", crate::core::Easing::OutQuint),
            ("back_out", crate::core::Easing::OutBack),
            ("elastic_out", crate::core::Easing::OutElastic),
        ] {
            check_text_fade(constructor, old, new, name, easing);
        }
    }
}

fn check_text_fade(
    constructor: &str,
    old: &str,
    new: &str,
    easing_name: &str,
    easing: crate::core::Easing,
) {
    use crate::core::{
        components::Draw2D,
        objects::{Latex2DShape, TextShape},
    };

    let source = r#"
return function(s)
    local label=s:CONSTRUCTOR{id='label',text=OLD,size=32,opacity=0.8}
    s:world_2d():add(label)
    local back=label:fade(OLD)
    s:wait(1)
    label:fade(NEW):play(2,k.ease.EASING)
    assert(label:get('text')==NEW)
    s:wait(1)
    back:play(2,k.ease.EASING)
end
"#
    .replace("CONSTRUCTOR", constructor)
    .replace("EASING", easing_name)
    .replace("OLD", &serde_json::to_string(old).unwrap())
    .replace("NEW", &serde_json::to_string(new).unwrap());
    let scene = load(&source);
    let label = scene.entity("label").unwrap();

    let progress = |time| easing.evaluate(time).clamp(0.0, 1.0);

    for (time, left_opacity, text) in [
        (0.0, 1.0, old),
        (1.0, 1.0, old),
        (1.5, 1.0 - progress(0.25), old),
        (2.0, 1.0 - progress(0.5), old),
        (2.5, 1.0 - progress(0.75), old),
        (2.9999, 1.0 - progress((2.9999 - 1.0) / 2.0), old),
        (3.0, 0.0, new),
        (4.5, progress(0.25), new),
        (6.0, 1.0, old),
        (1.5, 1.0 - progress(0.25), old),
    ] {
        let left_alpha = (204.0 * left_opacity).round() as i32;
        let right_alpha = (204.0 * (1.0 - left_opacity)).round() as i32;
        scene.update(time).unwrap();

        let world = scene.scene().world();
        let draw = world.get::<&Draw2D>(label).unwrap();
        let mut surface = skia_safe::surfaces::raster_n32_premul((640, 160)).unwrap();
        let canvas = surface.canvas();

        canvas.clear(skia_safe::colors::TRANSPARENT);
        canvas.translate((320.0, 80.0));
        (draw.on_draw)(&world, label, canvas, draw.opacity);

        let pixels = surface.peek_pixels().unwrap();
        let max_alpha = |start, end| {
            (0..160)
                .flat_map(|y| (start..end).map(move |x| (x, y)))
                .map(|point| i32::from(pixels.get_color(point).a()))
                .max()
                .unwrap()
        };

        assert!(
            (max_alpha(0, 320) - left_alpha).abs() <= 1,
            "Object: {constructor}, easing: {easing_name}, time: {time}."
        );
        assert!(
            (max_alpha(320, 640) - right_alpha).abs() <= 1,
            "Object: {constructor}, easing: {easing_name}, time: {time}."
        );
        let current = if constructor == "text_2d" {
            world.get::<&TextShape>(label).unwrap().text.clone()
        } else {
            world.get::<&Latex2DShape>(label).unwrap().text.clone()
        };

        assert_eq!(current, text);
        assert!((draw.opacity - 0.8).abs() < 0.001);
    }
}

#[test]
fn latex_3d_fade_tracks_formulas_progress_and_bounds() {
    use crate::core::{
        components::{Draw3D, Material},
        objects::{Latex3DShape, TextFade},
    };

    let scene = load(
        r#"
return function(s)
    local formula=s:latex_3d{id='formula',text='x',opacity=0.8,depth=0.25}
    local target=s:latex_3d{id='target',text='x+x+x',depth=0.25}
    s:world_3d():add(formula)
    formula:fade('x+x+x'):play(2,k.ease.back_out)
    assert(formula:get('text')=='x+x+x')
    s:wait(1)
    formula:fade('z'):play(0)
end
"#,
    );
    let formula = scene.entity("formula").unwrap();
    let target = scene.entity("target").unwrap();

    for (time, text, progress) in [
        (0.0, "x", 0.0),
        (0.8, "x", 1.0),
        (1.0, "x", 1.0),
        (2.0, "x+x+x", 1.0),
        (3.0, "z", 1.0),
        (0.8, "x", 1.0),
        (1.0, "x", 1.0),
    ] {
        scene.update(time).unwrap();

        let world = scene.scene().world();
        let fade = world.get::<&TextFade>(formula).unwrap();

        assert_eq!(world.get::<&Latex3DShape>(formula).unwrap().text, text);
        assert!((fade.progress - progress).abs() < 0.001);
        assert_eq!(fade.active, time < 2.0);
        assert_eq!(fade.incoming_text(text), (time < 2.0).then_some("x+x+x"));
        assert!((world.get::<&Material>(formula).unwrap().opacity - 0.8).abs() < 0.001);

        if time < 3.0 {
            let draw = world.get::<&Draw3D>(formula).unwrap();
            let target_draw = world.get::<&Draw3D>(target).unwrap();

            assert_eq!(
                (draw.box_size)(&world, formula),
                (target_draw.box_size)(&world, target)
            );
        }
    }

    assert!(try_load("return function(s) s:latex_2d{}:fade(123) end").is_err());
    assert!(try_load("return function(s) s:latex_3d{}:fade(123) end").is_err());
}

#[test]
fn callbacks_snapshot_camera_and_native_methods_return_definitions() {
    let scene = load(
        r#"
local Counter=k.object{
    name='Counter',components={k.component{value={default=0}}},
    methods={increment=function(self,amount) return self:by{value=amount} end},
}
return function(s)
    local ball=s:circle{id='ball',radius=10}
    local counter=s:object(Counter,{id='counter'})
    local label=s:text_2d{id='label',text='Old',opacity=0.8}
    s:world_2d():add(ball);s:world_2d():add(counter);s:world_2d():add(label)
    local snap=ball:snapshot();ball:save()
    local fade=label:fade('New')
    assert(label:get('text')=='Old')
    local change=counter:increment(10)
    assert(counter:get('value')==0 and counter:type_name()=='Counter')
    s:all(function()
        change:play(2,k.ease.linear)
        fade:play(1,k.ease.linear)
        ball:from{position={k.vec2(0,0),k.vec2(10,20)}}:play(1)
    end)
    assert(label:get('text')=='New')
    s:chain(function() ball:by{radius=10}:play(1);ball:restore(snap):play(1) end)
    ball:restore():play(0)
    ball:reset():play()
    local camera=s:world_3d():focus_camera{position=k.vec3(0,2,7),target=k.vec3(0,0,0)}
    s:parallel(function() camera:play(2) end)
    s:view('3d')
end
"#,
    );

    scene.update(0.5).unwrap();

    let label = scene.entity("label").unwrap();
    let world = scene.scene().world();

    assert_eq!(
        world
            .get::<&crate::core::objects::TextShape>(label)
            .unwrap()
            .text,
        "Old"
    );

    assert!(
        (world
            .get::<&crate::core::components::Draw2D>(label)
            .unwrap()
            .opacity
            - 0.8)
            .abs()
            < 0.001
    );

    drop(world);
    assert_eq!(scene.duration(), 4.0);
    scene.update(4.0).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(scene.entity("ball").unwrap())
            .unwrap()
            .radius,
        10.0
    );

    for source in [
        "local O=k.object{methods={to=function() end}};return function(s) end",
        "return function(s) s:text_2d{}:fade(123) end",
        "k.vec2=function() end;return function(s) end",
        "rawset(k,'vec2',false);return function(s) end",
        "k.ease.linear='Bad';return function(s) end",
        "setmetatable(k,{});return function(s) end",
    ] {
        assert!(
            try_load(source).is_err(),
            "Accepted invalid API mutation or method: {source}"
        );
    }
}

#[test]
fn bounds_channels_mesh_indices_and_orthogonal_height_are_typed() {
    let scene = load(
        r#"
local O=k.object{bounds=function() return {x=10,y=20,width=30,height=40} end}
return function(s)
    local o=s:object(O,{id='object'})
    local bounds=o:bounds();local size=o:box_size()
    assert(bounds.x==10 and bounds.y==20 and bounds.width==30 and size.x==30 and size.y==40)
    assert(k.quad(1,2,3,4).b==2 and math.abs(k.color(0.1,0.2,0.3,0.4).b-0.3)<0.00001)
    s:canvas_3d{camera_fov=10,camera_mode='Orthogonal'}:to{camera_fov=20}:play(1)
    k.mesh{vertices={{0,0,0},{1,0,0},{0,1,0}},indices={1,2,3}}
end
"#,
    );

    assert!(try_load("return function(s) s:world_3d():to{camera_fov=10}:play(1) end").is_err());
    assert_eq!(scene.scene().name(), "intro");

    assert!(
        try_load(
            "return function(s) k.mesh{vertices={{0,0,0},{1,0,0},{0,1,0}},indices={0,1,2}} end"
        )
        .is_err()
    );
}

#[test]
fn simulation_modules_inputs_and_reload_keep_explicit_deterministic_state() {
    let script = r#"
local definitions=require('objects.counter')
return function(s)
    local ball=s:circle{id='ball'}
    local counter=s:object(definitions,{id='counter',speed=3})
    local channel=s:object(k.object{
        state={x=0},inputs={},
        update=function(state,frame,inputs) state.x=inputs.x end,
    },{id='channel'})
    channel:inputs{x=ball:prop('position.x')}
    local empty=s:object(k.object{inputs={},update=function(state,frame,inputs) assert(next(inputs)==nil) end},{})
    s:world_2d():add(ball);s:world_2d():add(counter);s:world_2d():add(channel);s:world_2d():add(empty)
    ball:to{position={x=10}}:play(1,k.ease.linear)
    counter:signal(function(self) assert(self:state().result==6) end)
    channel:signal(function(self) assert(math.abs(self:state().x-ball:get('position.x'))<0.001) end)
end
"#;
    let mut sources = sources(script);

    sources.insert(
        "utils/math.lua".into(),
        "return {double=function(value) return value*2 end}".into(),
    );

    sources.insert(
        "objects/counter.lua".into(),
        r#"
return k.object{
    components={k.component{speed={default=1}}},state={result=0},
    update=function(state,frame,inputs)
        local helpers=require('utils.math')
        state.result=helpers.double(inputs.speed)
    end,
}
"#
        .into(),
    );

    let mut scene = LuaScene::load(
        ProjectContext::new(".").unwrap(),
        "intro.lua",
        sources.clone(),
        (640, 480),
        60,
        (0, 0),
    )
    .unwrap();

    scene.update(1.0).unwrap();
    scene.update(0.5).unwrap();
    scene.update(1.0).unwrap();

    let entity = scene.entity("counter").unwrap();
    let changed_script = script.replace("self:state().result==6", "self:state().result==9");

    sources.insert("intro.lua".into(), changed_script);

    sources.insert(
        "utils/math.lua".into(),
        "return {double=function(value) return value*3 end}".into(),
    );

    scene.reload(sources.clone()).unwrap();
    scene.update(0.5).unwrap();

    sources.insert(
        "utils/math.lua".into(),
        "local hidden=0;return {double=function(value) hidden=hidden+1;return value*2 end}".into(),
    );

    assert!(scene.reload(sources).is_err());
    assert_eq!(scene.entity("counter"), Some(entity));
    scene.update(0.5).unwrap();
}

#[test]
fn uniform_destinations_do_not_collide_with_object_properties() {
    let scene = load(
        r#"
local Mesh=k.object{dimension=3,components={k.component{pulse={default=2}}}}
return function(s)
    local mesh=s:object(Mesh,{
        id='mesh',shader=k.mesh_shader{fragment='uniform float pulse;out vec4 outColor;void main(){outColor=vec4(pulse);}'},uniforms={pulse=0},
    })
    s:world_3d():add(mesh)
    s:all(function()
        mesh:from{uniforms={pulse={0,6}}}:play(3,k.ease.linear)
        mesh:to{pulse=10,position={x=2}}:play(3,k.ease.linear)
    end)
    local bounds=mesh:bounds()
    assert(bounds.width==0)
end
"#,
    );

    scene.update(1.5).unwrap();

    let world = scene.scene().world();
    let entity = scene.entity("mesh").unwrap();

    assert_eq!(
        crate::core::objects::shader_uniform(&world, entity, "pulse").unwrap(),
        crate::core::TrackValue::F32(3.0)
    );

    assert_eq!(
        world.get::<&crate::core::Properties>(entity).unwrap().0["pulse"].1,
        crate::core::TrackValue::F32(6.0)
    );
}

#[test]
fn snapshots_capture_the_scheduling_position_without_changing_builder_values() {
    let scene = load(
        r#"
return function(s)
    local ball=s:circle{id='ball',radius=10}
    s:world_2d():add(ball)
    ball:to{radius=30}:play(1,k.ease.linear)
    assert(ball:get('radius')==30)
    ball:save()
    ball:to{radius=50}:play(1,k.ease.linear)
    local restore=ball:restore()
    assert(ball:get('radius')==50)
    restore:play(1,k.ease.linear)
end
"#,
    );

    scene.update(2.5).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(scene.entity("ball").unwrap())
            .unwrap()
            .radius,
        40.0
    );

    scene.update(0.0).unwrap();

    assert_eq!(
        scene
            .scene()
            .world()
            .get::<&CircleShape>(scene.entity("ball").unwrap())
            .unwrap()
            .radius,
        10.0
    );
}

#[test]
fn text_and_latex_morph_schedule_reusable_tasks_and_seek_in_both_dimensions() {
    use crate::core::objects::{Latex2DShape, Latex3DShape, Text3DShape, TextFade, TextShape};
    for (constructor, dimension) in [
        ("text_2d", "2d"),
        ("text_3d", "3d"),
        ("latex_2d", "2d"),
        ("latex_3d", "3d"),
    ] {
        let source = format!(
            r#"
return function(s)
    local label=s:{constructor}{{id='label',text='aba'}}
    s:world_{dimension}():add(label)
    local back=label:morph('aba')
    s:wait(1)
    label:morph('aac'):play(2,k.ease.linear)
    assert(label:get('text')=='aac')
    s:wait(1)
    back:play(2,k.ease.linear)
end
"#
        );
        let scene = load(&source);
        let label = scene.entity("label").unwrap();
        for (time, text, active, progress) in [
            (0.0, "aba", true, 0.0),
            (1.5, "aba", true, 0.25),
            (3.0, "aac", false, 1.0),
            (4.5, "aac", true, 0.25),
            (6.0, "aba", false, 1.0),
            (2.0, "aba", true, 0.5),
        ] {
            scene.update(time).unwrap();
            let world = scene.scene().world();
            let fade = world.get::<&TextFade>(label).unwrap();
            assert_eq!(fade.active, active, "{constructor} at {time}.");
            assert!((fade.progress - progress).abs() < 0.001);
            if active {
                assert!(fade.morph);
            }
            let current = match constructor {
                "text_2d" => world.get::<&TextShape>(label).unwrap().text.clone(),
                "text_3d" => world.get::<&Text3DShape>(label).unwrap().text.clone(),
                "latex_2d" => world.get::<&Latex2DShape>(label).unwrap().text.clone(),
                _ => world.get::<&Latex3DShape>(label).unwrap().text.clone(),
            };
            assert_eq!(current, text);
        }
        assert!(
            try_load(&format!(
                "return function(s) s:{constructor}{{}}:morph(123) end"
            ))
            .is_err()
        );
    }
}

#[test]
fn morph_moves_shared_glyphs_without_crossfading_them() {
    use crate::core::components::Draw2D;
    for (constructor, old, new) in [
        ("text_2d", "I        ", "        I"),
        ("latex_2d", r"x\qquad\qquad", r"\qquad\qquad x"),
    ] {
        let source = format!(
            "return function(s) local t=s:{constructor}{{id='label',text={},size=32,opacity=0.8}} s:world_2d():add(t) t:morph({}):play(2,k.ease.linear) end",
            serde_json::to_string(old).unwrap(),
            serde_json::to_string(new).unwrap(),
        );
        let scene = load(&source);
        let label = scene.entity("label").unwrap();
        let render = |time| {
            scene.update(time).unwrap();
            let world = scene.scene().world();
            let draw = world.get::<&Draw2D>(label).unwrap();
            let mut surface = skia_safe::surfaces::raster_n32_premul((640, 160)).unwrap();
            let canvas = surface.canvas();
            canvas.clear(skia_safe::colors::TRANSPARENT);
            canvas.translate((320.0, 80.0));
            (draw.on_draw)(&world, label, canvas, draw.opacity);
            let pixels = surface.peek_pixels().unwrap();
            let mut left = 640;
            let mut right = 0;
            let mut alpha = 0;
            for y in 0..160 {
                for x in 0..640 {
                    let value = pixels.get_color((x, y)).a();
                    if value > 0 {
                        left = left.min(x);
                        right = right.max(x);
                    }
                    alpha = alpha.max(value);
                }
            }
            ((left + right) as f32 * 0.5, alpha)
        };
        let start = render(0.0);
        let end = render(2.0);
        let middle = render(1.0);
        assert!(
            (middle.0 - (start.0 + end.0) * 0.5).abs() <= 1.0,
            "{constructor}."
        );
        assert!((i32::from(middle.1) - 204).abs() <= 1, "{constructor}.");
        assert!((start.0 - end.0).abs() > 20.0);
        assert_eq!(middle, render(1.0));
        assert_eq!(start, render(0.0));
    }
}

#[test]
fn reparent_moves_subtrees_and_preserves_lifetimes() {
    let scene = load(
        r#"
return function(s)
    local left = s:group_2d {id='left', position=k.vec2(10, 0)}
    local right = s:group_2d {id='right', position=k.vec2(20, 0)}
    s:world_2d():add(left)
    s:world_2d():add(right)
    local ball = s:circle {id='ball', position=k.vec2(3, 0)}
    s:wait(1)
    left:add(ball)
    s:wait(1)
    ball:remove()
    s:wait(1)
    ball:reparent(right)
    ball:reparent(right)
    assert(#left:children() == 0 and #right:children() == 1)
    assert(ball:global_position().x == 23 and ball.position:get().x == 3)
    left:reparent(right)
    assert(#right:children() == 2 and #s:world_2d():children() == 1)
    local function rejects(callback, message)
        local ok, err = pcall(callback)
        assert(not ok and string.find(tostring(err), message, 1, true))
    end
    rejects(function() right:reparent(left) end, 'cycle')
    rejects(function() right:reparent(right) end, 'itself')
    rejects(function() ball:reparent(s:world_3d()) end, 'Cannot mix 2D and 3D')
    rejects(function() ball:reparent(s:root()) end, 'Only canvases')
    assert(#right:children() == 2 and #left:children() == 0)
    local inactive = s:circle {id='inactive'}
    inactive:reparent(left)
    assert(#left:children() == 1)
end
"#,
    );
    let world = scene.scene().world();
    let ball = world
        .get::<&crate::core::components::TreeNode>(scene.entity("ball").unwrap())
        .unwrap();
    assert_eq!(ball.lifetime, [1.0, 2.0]);
    let inactive = world
        .get::<&crate::core::components::TreeNode>(scene.entity("inactive").unwrap())
        .unwrap();
    assert_eq!(inactive.lifetime, [3.0, f32::INFINITY]);
}

#[test]
fn sound_resources_schedule_without_advancing_or_mutating_defaults() {
    let scene = load(
        r#"
return function(s)
    local unused=s:sound{preset='whoosh'}
    local boink=s:sound{preset='boink',gain=0.2}
    boink:play()
    boink:play{frequency=600,gain=0.15}
    s:wait(0.5)
    boink:play()
    s:all(function(s) s:wait(1); boink:play() end)
    s:wait(0.5)
end
"#,
    );
    assert_eq!(scene.duration(), 2.0);
    let audio = scene.scene().audio.borrow();
    assert_eq!(
        audio
            .scheduled
            .iter()
            .map(|event| event.time)
            .collect::<Vec<_>>(),
        [0.0, 0.0, 0.5, 0.5]
    );
    assert_eq!(audio.scheduled[0].samples, audio.scheduled[2].samples);
    assert_ne!(audio.scheduled[0].samples, audio.scheduled[1].samples);
}

#[test]
fn simulation_sounds_survive_seek_checkpoints_and_reload_without_duplicates() {
    let script = r#"
return function(s)
    local hit=s:sound{preset='click'}
    local simulation=k.object{state={n=0},inputs={},update=function(state,frame,inputs)
        state.n=state.n+1
        if frame.index%30==0 then
            inputs.hit:play()
            inputs.hit:play{pan=0.5}
        end
    end}
    local object=s:object(simulation,{id='simulation'})
    object:inputs{hit=hit}
    s:world_2d():add(object)
    s:wait(5)
end
"#;
    let mut scene = load(script);
    scene.update(4.5).unwrap();
    let mix = |scene: &LuaScene| {
        let mut output = vec![[0.0; 2]; 240_000];
        scene.scene().audio.borrow().mix(0.0, 0, &mut output);
        output
    };
    let expected = mix(&scene);
    let event_count = |scene: &LuaScene| {
        scene
            .scene()
            .audio
            .borrow()
            .steps
            .values()
            .map(Vec::len)
            .sum::<usize>()
    };
    assert_eq!(event_count(&scene), 20);
    scene.update(0.1).unwrap();
    scene.update(4.5).unwrap();
    assert_eq!(event_count(&scene), 20);
    assert_eq!(mix(&scene), expected);
    scene
        .reload(sources(&format!("-- Layout change.\n{script}")))
        .unwrap();
    assert_eq!(event_count(&scene), 20);
    assert_eq!(mix(&scene), expected);
    let candidate = scene.prepare_seek().unwrap();
    candidate.update(4.5).unwrap();
    assert_eq!(event_count(&candidate), 20);
    assert_eq!(mix(&candidate), expected);
    scene
        .reload(sources(
            &script.replace("preset='click'", "preset='impact'"),
        ))
        .unwrap();
    scene.update(4.5).unwrap();
    assert_eq!(event_count(&scene), 20);
    assert_ne!(mix(&scene), expected);
}

#[test]
fn explicit_simulation_updates_at_the_same_step_keep_each_sound() {
    let scene = load(
        r#"
return function(s)
    local sound=s:sound{preset='click'}
    local object=s:object(k.object{inputs={},update=function(state,frame,inputs) inputs.sound:play() end},{auto_update=false})
    object:inputs{sound=sound}
    s:world_2d():add(object)
    object:update(); object:update(); object:checkpoint()
    s:wait(1)
end
"#,
    );
    scene.update(0.0).unwrap();
    assert_eq!(
        scene
            .scene()
            .audio
            .borrow()
            .steps
            .values()
            .map(Vec::len)
            .sum::<usize>(),
        2
    );
    scene.update(0.5).unwrap();
    scene.update(0.0).unwrap();
    assert_eq!(
        scene
            .scene()
            .audio
            .borrow()
            .steps
            .values()
            .map(Vec::len)
            .sum::<usize>(),
        2
    );
}

#[test]
fn sound_validation_and_failed_simulation_leave_no_partial_events() {
    for options in [
        "{preset='missing'}",
        "{gain=0/0}",
        "{pan=2}",
        "{frequency=-1}",
        "{waveform='square'}",
        "{attack=10,release=10}",
        "{typo=1}",
    ] {
        assert!(
            LuaScene::load(
                ProjectContext::new(".").unwrap(),
                "intro.lua",
                sources(&format!("return function(s) s:sound{options} end")),
                (640, 480),
                60,
                (0, 0)
            )
            .is_err()
        );
    }
    assert!(LuaScene::load(
        ProjectContext::new(".").unwrap(), "intro.lua",
        sources("return function(s) local sound=s:sound{attack=1,decay=1,hold=7,release=1}; for i=1,20 do sound:play() end end"),
        (640,480),60,(0,0)
    ).err().unwrap().to_string().contains("sample budget"));
    let scene = load(
        r#"
return function(s)
    local sound=s:sound{preset='click'}
    local object=s:object(k.object{inputs={},update=function(state,frame,inputs)
        inputs.sound:play()
        if frame.index==10 then error('Failed step.') end
    end},{})
    object:inputs{sound=sound}
    s:world_2d():add(object)
    s:wait(1)
end
"#,
    );
    scene.update(0.0).unwrap();
    let count = scene.scene().audio.borrow().steps.len();
    assert!(scene.update(0.5).is_err());
    assert_eq!(scene.scene().audio.borrow().steps.len(), count);
}

#[test]
fn sound_tails_extend_the_scene_without_advancing_the_animator() {
    assert_eq!(load("return function(s) s:sound{} end").duration(), 0.0);
    let scene = load(
        "return function(s) local tone=s:sound{waveform='triangle',frequency=220,attack=0.01,decay=0.02,hold=0.03,release=0.04}; tone:play(); tone:play(); s:wait(0.01); tone:play() end",
    );
    let audio = scene.scene().audio.borrow();
    assert_eq!(audio.scheduled[0].time, 0.0);
    assert_eq!(audio.scheduled[1].time, 0.0);
    assert!((audio.scheduled[2].time - 0.01).abs() < 1e-6);
    assert!((scene.duration() - 0.11).abs() < 0.0001);
}
