use kinematic::{
    core::{ProjectContext, components::Transform2D, objects::CircleShape},
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
local k = require('kinematic')
return k.scene('Intro', function(s)
    local ball = s:circle {id='ball', radius=24, fill=k.color('#60A5FA'), position=k.vec2(-100, 0)}
    s:world_2d():add(ball)
    s:play(ball.position:to(k.vec2(100, 0), 2, k.ease.linear))
    s:wait(1)
end)
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
local k = require('kinematic')
return k.scene('Spatial', function(s)
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
        s:play(a.opacity:to(1, 1, k.ease.linear))
        s:play(a.scale:to(k.vec2(1, 1), 1, k.ease.linear))
    end)
    assert(not pcall(function() return a.fade_in end))
    assert(not pcall(function() return a.grow_in end))
    s:wait(1)
end)
"#,
    );
    scene.update(0.5).unwrap();
    let world = scene.scene().world();
    let a = scene.entity("a").unwrap();
    assert_eq!(world.get::<&Transform2D>(a).unwrap().scale.x, 0.5);
    assert_eq!(
        world
            .get::<&kinematic::core::components::Draw2D>(a)
            .unwrap()
            .opacity,
        0.5
    );
}

#[test]
fn removing_a_shader_on_reload_removes_its_components() {
    use kinematic::core::objects::MeshShaderData;
    let source = r#"
local k = require('kinematic')
return k.scene('Shader', function(s)
    local shape = s:sphere {id='shape', shader=k.mesh_shader {fragment='out vec4 outColor; void main() { outColor=vec4(1.0); }'}}
    s:world_3d():add(shape)
    s:wait(1)
end)
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
local k = require('kinematic')
return k.scene('Transaction', function(s)
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
    s:play(a.position:to(k.vec2(100, 0), 2, k.ease.linear))
end)
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
            .get::<&kinematic::core::components::Draw2D>(b)
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
local k=require('kinematic')
return k.scene('Spin', function(s)
    local shape=s:sphere {id='spin'}
    s:world_3d():add(shape)
    s:play(shape.rotation:rotate(k.vec3(0,0,1), 2*math.pi, 1, k.ease.linear))
end)
"#,
    );
    scene.update(0.25).unwrap();
    let world = scene.scene().world();
    let transform = world
        .get::<&kinematic::core::components::Transform3D>(scene.entity("spin").unwrap())
        .unwrap();
    assert!(
        (transform.rotation * kinematic::glam::Vec3::X - kinematic::glam::Vec3::Y).length() < 0.001
    );
}

#[derive(Default, kinematic::prelude::Object)]
#[object(spatial = "2d", builder = "host_dot")]
struct HostDot {
    #[trackable]
    shape: CircleShape,
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: kinematic::core::components::Draw2D,
}

#[test]
fn registered_native_objects_share_generated_fields_and_definitions() {
    use kinematic::scripting::LuaProject;
    let root =
        std::env::temp_dir().join(format!("kinematic-native-extension-{}", std::process::id()));
    let mut project = LuaProject::create(&root, "Native extension", false).unwrap();
    project.register::<HostDot>().unwrap();
    assert!(project.register::<HostDot>().is_err());
    assert!(project.definitions().contains("name: 'host_dot'"));
    assert!(
        project
            .definitions()
            .contains("---@class Kinematic.HostDot")
    );
    let source = [(PathBuf::from("scenes/intro.lua"), "local k=require('kinematic'); return k.scene('Host',function(s) local dot=s:native('host_dot',{id='dot',radius=7}); s:world_2d():add(dot); s:wait(1) end)".into())].into();
    let scenes = project.load_scenes(&source).unwrap();
    let entity = scenes[0].entity("dot").unwrap();
    assert_eq!(
        scenes[0]
            .scene()
            .world()
            .get::<&CircleShape>(entity)
            .unwrap()
            .radius,
        7.0
    );
    drop(scenes);
    std::fs::remove_dir_all(root).unwrap();
}
