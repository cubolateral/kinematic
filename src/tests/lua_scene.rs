use crate::{
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
    ball:from {position={x={0,200}},radius={16,39},opacity={0,1}}
        :to {position={y=50}}:play(2,k.ease.linear)
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
    local destination=ball:to{position={x=10}}
    local same=destination:to{radius=24}
    assert(rawequal(same,destination))
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
        "ball:from{position={x={0,10}}}:to{position={x=20}}",
        "ball:to{position={x=20}}:by{position=k.vec2(1,2)}",
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
        "New"
    );

    assert!(
        (world
            .get::<&crate::core::components::Draw2D>(label)
            .unwrap()
            .opacity
            - 0.4)
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
    mesh:from{uniforms={pulse={0,6}}}:to{pulse=10,position={x=2}}:play(3,k.ease.linear)
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
