use crate::{core::ProjectContext, scripting::LuaScene};
use std::{hint::black_box, path::PathBuf, time::Instant};

fn load(source: &str) -> LuaScene {
    LuaScene::load(
        ProjectContext::new(".").unwrap(),
        "benchmark.lua",
        [(PathBuf::from("benchmark.lua"), source.to_owned())].into(),
        (640, 480),
        60,
        (0, 0),
    )
    .unwrap()
}

fn measure(name: &str, iterations: usize, mut run: impl FnMut()) {
    if std::env::var("KINEMATIC_BENCH_FILTER").is_ok_and(|filter| !name.contains(&filter)) {
        return;
    }
    for _ in 0..3 {
        run();
    }
    let mut samples = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        for _ in 0..iterations {
            run();
        }
        samples.push(start.elapsed().as_secs_f64() * 1e6 / iterations as f64);
    }
    samples.sort_by(f64::total_cmp);
    println!(
        "{name},{:.3},{:.3},{:.3}",
        samples[3], samples[0], samples[6]
    );
}

fn objects(kind: &str, behavior: &str) -> String {
    format!(
        "local k=require('kinematic'); return k.scene('Bench',function(s) \
         s:all(function() for i=1,1000 do \
         local obj=s:{kind}{{id='o'..i}}; \
         s:world_{}():add(obj); {behavior} end end); s:wait(10) end)",
        if kind == "sphere" { "3d" } else { "2d" }
    )
}

#[test]
#[ignore = "Runs the Lua performance benchmark."]
fn lua_benchmark() {
    println!("case,median_us,min_us,max_us");
    let static_source = objects("circle", "");
    measure("load_1000_2d", 5, || {
        black_box(load(&static_source));
    });
    for (name, source) in [
        ("static_1000", static_source.clone()),
        (
            "tracks_1000_2d",
            objects("circle", "s:play(obj.radius:to(20,10,k.ease.linear))"),
        ),
        (
            "tracks_1000_3d",
            objects("sphere", "s:play(obj.radius:to(20,10,k.ease.linear))"),
        ),
        (
            "signals_1000",
            objects(
                "circle",
                "obj:signal(function(o,f) o.radius:set(10+f.time) end)",
            ),
        ),
    ] {
        let scene = load(&source);
        assert!(scene.entity("o1000").is_some());
        let mut frame = 0;
        measure(name, 60, || {
            frame = (frame + 1) % 600;
            scene.update(black_box(frame as f32 / 60.0)).unwrap();
        });
    }
    let life = include_str!("../../examples/lua/simulation/scenes/life.lua");
    {
        let source = format!(
            "local k=require('kinematic'); local cache={{}} \
             for i=1,2000 do cache[i]=i*i end \
             local obj=k.object{{state={{total=0,cache=cache}},inputs={{}}, \
             update=function(state) state.total=state.total+1 end}}; \
             return k.scene('Cache',function(s) local o=s:object(obj,{{id='cache'}}); \
             s:world_2d():add(o); s:wait(30) end)"
        );
        let scene = load(&source);
        let mut frame = 0;
        measure("cache_full_step", 60, || {
            frame += 1;
            scene.update(black_box(frame as f32 / 60.0)).unwrap();
        });
        measure("cache_full_seek", 10, || {
            scene.update(2.0).unwrap();
            scene.update(4.0).unwrap();
        });
    }
    let scene = load(life);
    let mut frame = 0;
    measure("life_step", 120, || {
        frame = (frame + 1) % 180;
        scene.update(black_box(frame as f32 / 60.0)).unwrap();
    });
    measure("life_seek", 10, || {
        scene.update(0.0).unwrap();
        scene.update(1.5).unwrap();
    });
    let mut scene = load(life);
    scene.update(2.5).unwrap();
    measure("life_reload", 5, || {
        scene
            .reload([(PathBuf::from("benchmark.lua"), life.to_owned())].into())
            .unwrap();
    });
    let scene = load(life);
    let mut surface = skia_safe::surfaces::raster_n32_premul((640, 480)).unwrap();
    measure("life_draw_raster", 60, || {
        surface.canvas().clear(skia_safe::colors::TRANSPARENT);
        scene.scene().draw(surface.canvas());
        black_box(&mut surface);
    });
}
