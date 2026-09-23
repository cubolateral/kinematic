# Kinematic

Kinematic is a Rust animation editor and library built around an ECS scene,
typed animation tracks, and a timeline-based workflow. It uses
[SDL3](https://github.com/libsdl-org/SDL) and OpenGL for the application runtime,
[Skia](https://skia.org/) for 2D rendering,
[three-d](https://github.com/asny/three-d) for 3D rendering, and
[Dear ImGui](https://github.com/ocornut/imgui) for the editor UI, and
[FFmpeg](https://ffmpeg.org/) as its video exporter.

Kinematic is in early development, so its API may change.

## Features

- Typed scene objects and trackable component fields.
- Invisible typed tracks for small scene-local animated values.
- First-class 2D and 3D canvases rendered with Skia and three-d, respectively.
- User-facing object names with type-based defaults.
- Hierarchical scene trees with reusable 2D and 3D containers and inherited transforms.
- Animatable orthographic 2D and perspective 3D cameras.
- Built-in and custom `Draw3D` meshes with reusable geometry caches, materials, and lighting.
- Builder-configured GLSL image and mesh shaders.
- Bidirectional projection of 2D and 3D canvases.
- Sequential and parallel animation tasks.
- Named, editable event waits persisted per scene.
- Reactive signals that run after tracks and can temporarily override properties.
- Deterministic random generation with state-independent keyed forks.
- Deterministic frame-dependent simulations with seekable runtime checkpoints.
- Sequential multi-scene projects.
- Built-in easing functions.
- Hierarchical timeline and selection from the Scene Tree, Timeline, or Preview.
- SDL3/OpenGL rendering with an internal Dear ImGui editor.
- FFmpeg-backed MP4 export from the editor. Exported projects are
  encoded as MP4 files in the `output/` directory.

## Requirements

- A Rust toolchain with edition 2024 support.
- CMake to build the bundled SDL3 source.
- A native C/C++ toolchain to compile SDL3 and Dear ImGui.
- A desktop environment with OpenGL 3.3 support.
- FFmpeg available in `PATH` for MP4 export.

## Example

```rust
use kinematic::prelude::*;

#[scene]
fn example(s: &mut Scene) {
    let circle = circle()
        .radius(128.0)
        .position(vec2(-256.0, 0.0))
        .fill(Color::RED)
        .build(s);

    s.world_2d().add(&circle);

    circle
        .position_x(256.0)
        .fill(Color::BLUE)
        .play();

    s.wait(1.0);
}

fn main() {
    app().project("Example", vec![example]).run();
}
```

Scene factories run in vector order. Each scene starts as soon as the previous
scene reaches the end of its timeline. Resolution and frame rate default to
1280 x 720 and 60 FPS. Change them in the editor's Configuration panel;
Kinematic stores them in `.kinematic/project.ron`.

## Timed events

Use `event` for a named wait whose duration can be edited directly in the
Timeline:

```rust
#[scene]
fn introduction(s: &mut Scene) {
    s.event("intro");
    // Objects, animations, signals, creation, and removal scheduled here start
    // after the editable intro duration.
}
```

Kinematic stores the durations for this scene in
`.kinematic/scenes/introduction.ron`:

```ron
(
    events: [
        (name: "intro", duration: 2.0),
    ],
)
```

An event behaves like `wait(duration)` at the point where it appears. Its
Timeline span starts at that scheduling position. Drag the labeled handle at
the end of the span to change its duration; releasing it saves the RON file and
rebuilds only that scene. Events can be used with `chain`, `all`, signals, and
object lifetime changes, but not inside `repeat`.

## Animation groups and loops

`scene.chain(...)` schedules its children in sequence. `scene.all(...)` starts
all children at the group start, including object attachment and effects; its
duration is the longest child. Use a nested `chain` to delay a parallel branch.
Concurrent animations of the same property are rejected.

`scene.repeat(...)` describes one finite cycle and repeats it without advancing
the outer timeline. The closure runs once; playback and seeking evaluate the
stored cycle directly. For example, this scene levitates for ten seconds:

```rust
#[scene]
fn levitation(s: &mut Scene) {
    let object = circle().build(s);
    s.world_2d().add(&object);

    s.repeat(|_| {
        object.position_y(-40.0).duration(1.0).easing(Easing::InOutSine).play();
        object.position_y(0.0).duration(1.0).easing(Easing::InOutSine).play();
    });
    s.wait(10.0);
}
```

Cycles must contain animation and have a finite, positive duration. They may
contain `chain`, `all`, and waits, but cannot nest another `repeat` or create,
attach, or remove objects. Create objects before entering the cycle. A repeated
property cannot have another animation at or after the cycle starts, except an
instantaneous setup at its start. Other properties can animate independently.

Repetition ends visually with the scene or object's lifetime. A scene containing
only background cycles needs a finite duration, established with waits or other
animations. The Timeline shows the first cycle normally and repeats its segments
and keyframes with reduced opacity.

For a finite number of repetitions, use a Rust `for` loop. This replaces the old
`scene.repeat(count, ...)` API. The task equivalent is now `Task::Repeat(tasks)`.
Each iteration of a `for` constructs fresh animations and can create objects.

For continuous 3D spinning, repeat `object.rotation_y_by(TAU)` with linear easing.
The axis-angle path preserves direction and full turns; `object.rotation(q)`
interpolates orientations along the shortest quaternion path. Match the cycle's
end and start for a seamless loop; repetition does not automatically close it.

## Standalone tracks

Use `scene.track(initial)` when a small animated value does not belong to a
scene object or custom trackable component:

```rust
let counter = s.track(0_u32);

counter
    .to(10)
    .duration(2.0)
    .easing(Easing::Linear)
    .play();

let counter_for_signal = counter.clone();
object.signal(move |object, _frame| {
    object.set_sides(counter_for_signal.get());
});
```

The initial and target values are converted through `TrackValueType`; callers
do not manipulate `TrackValue` directly. Standalone tracks support normal
timeline groups, `repeat`, and seeking, but remain hidden from the editor UI.

## Deterministic random values

`Random` is a clonable generator independent of `Scene`. `Random::new(seed)`
uses a stable xoshiro256++ sequence with SplitMix64 seed mixing. A clone resumes
from the same state. `fork(key)` instead derives a child from the generator's
original seed, so consuming values from the parent does not change the child:

```rust
let mut random = Random::new(42);
let radius = random.range_f32(24.0..96.0);
let fill = *random.choose(&[Color::RED, Color::GREEN, Color::BLUE]).unwrap();

let first = random.fork(7).u64();
random.u64();
let replayed = random.fork(7).u64();
assert_eq!(first, replayed);
```

Primitive methods are named after their result type: `bool`, `u8`, `u16`,
`u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`,
`f32`, and `f64`. Integer and floating-point ranges use the explicit
`range_u32`, `range_u64`, `range_usize`, `range_i32`, `range_i64`,
`range_f32`, and `range_f64` methods. Ranges are half-open.

`chance`, `choose`, `choose_mut`, `choose_weighted`, `choose_multiple`,
`shuffle`, and `normal` cover common sampling operations. `choose_weighted`
receives a values slice and a matching `f64` weights slice. Invalid weights or
an empty input return `None`. The zero-argument `random()` convenience function
creates `Random::new(0)`.

## Simulations

`Simulation` is a component that any custom `Object` can own. Use it when the
next state depends on the previous frame, as in physics, cellular automata, and
procedural systems. The state advances in fixed steps with `dt = 1.0 / fps` and
must be `Clone + Send + Sync + 'static` so checkpoints can restore it on seek.

```rust
#[derive(Clone)]
struct Life {
    cells: [bool; 8],
}

impl SimulationState for Life {
    fn on_update(&mut self, _context: &SimulationContext<'_>) {
        // Advance one frame.
    }
}

impl SimulationState2D for Life {
    fn on_draw(&mut self, _world: &hecs::World, _entity: hecs::Entity, canvas: &skia_safe::Canvas) {
        // Draw the current cells in local coordinates.
    }
}

#[derive(Object)]
#[object(spatial = "2d", builder = "life", simulation = Life)]
struct LifeObject {
    #[trackable]
    simulation: Simulation,
    #[trackable]
    transform: Transform2D,
    #[trackable]
    draw: Draw2D,
}

impl Default for LifeObject {
    fn default() -> Self {
        Self {
            simulation: Simulation::new_2d(Life { cells: [false; 8] }),
            transform: Transform2D::default(),
            draw: Draw2D {
                on_draw: Simulation::draw_2d,
                box_size: Simulation::box_2d,
                visual_bounds: Simulation::visual_bounds_2d,
                ..Draw2D::default()
            },
        }
    }
}
```

The generated handler exposes `update`, `write_simulation`, and
`read_simulation`. Writes and explicit updates are stored in timeline order and
replayed through seeks and checkpoints:

```rust
let life = life().auto_update(false).build(s);
s.world_2d().add(&life);

life.write_simulation(|state| state.cells[2] = true);
life.update();
s.wait(1.0);
life.write_simulation(|state| state.cells[4] = false);
```

`read_simulation` reads the state produced by the latest `Scene::update` and
returns an owned result, so no reference escapes the ECS borrow:

```rust
let alive = life.read_simulation(|state| state.cells[2]);
```

Use a signal to derive another object's displayed value on every evaluated
frame. Signals run after simulations:

```rust
let label = text_2d().text("Alive: 0").build(s);
let label_for_signal = label.clone();

life.signal(move |life, _frame| {
    let alive = life.read_simulation(|state| state.cells.iter().filter(|cell| **cell).count());
    label_for_signal.set_text(format!("Alive: {alive}"));
});
```

Normal `#[trackable]` components can live beside `Simulation` and are available
through their generated builder and handler fields. During `on_update`, sample
their historical values with `SimulationContext::get::<T>()`. Use
`Simulation::new` for fully custom drawing, or `new_2d`/`new_3d` with the
optional `Simulation::draw_2d`/`draw_3d` callbacks.

A complete runnable version contains equivalent 2D and 3D scenes, including a
glider with automatic advancement disabled and a burst of explicit steps. It is in
[`examples/game_of_life.rs`](examples/game_of_life.rs). Run it with:

```sh
cargo run --example game_of_life
```

## Signals

Use `signal` to run logic after the scene's tracks have been evaluated. The
callback receives a fresh clone of the object handler and a `SignalFrame` on
every evaluation. Direct `set_*` methods temporarily override track values until
the next update:

```rust
#[scene]
fn follow_camera(s: &mut Scene) {
    let circle = circle().build(s);
    s.world_2d().add(&circle);

    s.world_2d().signal(move |handler, _frame| {
        handler.set_camera_position(circle.global_position());
    });
}
```

`SignalFrame` exposes the current scene `time`, the project-frame `index`, and
the fixed frame duration `dt`. Its index uses the scene FPS, making a keyed fork
stable when the timeline seeks or evaluates the same frame again:

```rust
let random = Random::new(42);

object.signal(move |object, frame| {
    let mut random = random.fork(frame.index);
    object.set_opacity(random.range_f32(0.5..1.0));
});
```

Signals begin at the current animator time and do not extend the scene
duration. Keep the returned `SignalHandle` and call `stop()` to end one at the
current animator time. Signals can be evaluated while seeking in either
direction, and are skipped while their target object is inactive.

When a callback uses another handler, that handler must be moved into the
stored callback because signals outlive the builder closure:

```rust
let circle = circle().build(s);
let circle_for_signal = circle.clone();

canvas.signal(move |handler, _frame| {
    handler.set_camera_position(circle_for_signal.global_position());
});
```

Signals cannot be created inside `repeat()`, and callbacks cannot alter the
scene structure or timeline.

## 2D and 3D rendering

Kinematic treats 2D and 3D as equal parts of the same animation model. Skia
renders vector-based `Canvas2D` content, while three-d renders depth-tested
`Canvas3D` content with perspective cameras, meshes, materials, and lighting.
Both dimensions use the same scene tree, typed tracks, timeline, seeking, and
MP4 export workflow.

Every scene includes `World 2D` and `World 3D` canvases at the project
resolution. Each canvas owns exactly one camera component, created with the
canvas and unavailable for removal. The 2D world is selected by default. A 3D
scene only needs objects and a change to the root's `view_2d` track; its camera
starts at `(0, 0, 3)`:

```rust
#[scene]
fn scene_3d(s: &mut Scene) {
    s.root().view_2d(false).immediate();

    let group = group_3d().build(s);
    s.world_3d().add(&group);

    group.add(
        &cube()
            .position(vec3(-1.0, 0.0, 0.0))
            .albedo(Color::BLUE)
            .build(s),
    );
    group.add(
        &sphere()
            .position(vec3(1.0, 0.0, 0.0))
            .albedo(Color::RED)
            .build(s),
    );

    group
        .rotation(Quaternion::from_rotation_y(PI))
        .duration(2.5)
        .play();

    s.wait(1.0);
}
```

`view_2d` is a discrete boolean track: `true` selects `World 2D` and `false`
selects `World 3D`. Schedule additional immediate changes to alternate between
them during the video. Three-dimensional position, quaternion rotation, scale,
camera perspective, and material properties can be animated through the same
typed builder and handler API used by 2D objects.

Rendering callbacks are dimension-specific components: `Draw2D` receives a
Skia canvas, while `Draw3D` receives a `RenderContext3D`. The 3D context exposes
the active camera, render target, three-d context, canvas textures, and a mesh
cache keyed by `GeometryKey`. This lets application-defined objects participate
in the same scene tree and reuse GPU geometry across objects and frames.
`#[derive(Object)]` also bundles the struct fields as ECS components; a separate
`#[derive(hecs::Bundle)]` is not needed:

```rust
use kinematic::{hecs, prelude::*, three_d};

#[derive(Clone)]
struct CustomShape;

#[derive(Object)]
#[object(spatial = "3d", builder = "custom_cube")]
struct CustomCube {
    #[trackable]
    transform: Transform3D,

    shape: CustomShape,
    draw: Draw3D,
}

impl Default for CustomCube {
    fn default() -> Self {
        Self {
            transform: Transform3D::default(),
            shape: CustomShape,
            draw: Draw3D {
                on_draw: draw_custom,
                box_size: |_, _| Vector3::ONE,
            },
        }
    }
}

fn draw_custom(
    world: &hecs::World,
    entity: hecs::Entity,
    context: &mut RenderContext3D<'_>,
) -> Result<(), String> {
    context.render_material(
        GeometryKey::new::<CustomShape>(0),
        three_d::CpuMesh::cube,
        global_matrix3d(world, entity),
        &Material::default(),
    )
}
```

The bounds callback returns the object's local size. A cache key must change
whenever the generated CPU geometry changes; transforms and material values do
not belong in the key.

### GLSL shaders

Shader definitions are reusable, while uniforms and texture bindings belong to
each built object. The shader remains builder-only; handlers can read, set, and
animate uniforms that were declared by that object's builder. Updating a
uniform does not recompile the shared GLSL program.

An image shader is a complete GLSL 3.30 fragment shader. It processes a 2D
object, a composed `group_2d`, or a 2D/3D canvas image:

```rust
let tint = ImageShader::new(r#"#version 330 core
in vec2 k_uv;
out vec4 k_color;
uniform sampler2D k_image;
uniform vec4 tint;
uniform float u_progress;

void main() {
    k_color = texture(k_image, k_uv) * tint * u_progress;
}
"#);

let object = circle()
    .shader(&tint)
    .uniform("tint", Color::RED)
    .uniform("u_progress", 0.0_f32)
    .shader_padding(8.0)
    .build(s);

let progress = object.get_uniform::<f32>("u_progress");
object.set_uniform("u_progress", progress + 0.1);
object
    .uniform("u_progress", 1.0_f32)
    .position_x(200.0)
    .duration(2.0)
    .play();
object
    .uniform_from("u_progress", 0.0_f32, 1.0_f32)
    .play();
```

The engine supplies `k_image`, pixel-sized `k_resolution`, evaluated scene
seconds in `k_time`, and `k_alpha`. UV `(0, 0)` is the bottom-left pixel. Input
and output are premultiplied, sRGB-encoded RGBA. Additional canvas samplers use
`.texture(name, &canvas)`; canvas dependencies are ordered and cycles are
rejected. Captures are allocated on demand, and `shader_padding` expands them
for blur or glow. CPU morph capture rejects shader-rendered objects because its
result depends on the GPU renderer.

Uniform tracks use the normal timeline, easing, repetition, seek, signals,
snapshots, Timeline, Inspector, and export evaluation. Supported values are
`bool`, `u32`, `i32`, `f32`, `Quad`, `Vector2`, `Vector3`, `Quaternion`, and
`Color`; enum and string tracks have no GLSL uniform representation. Canvas
texture bindings remain fixed builder values and are not interpolated.

A mesh shader either supplies only a fragment stage, paired with the standard
vertex stage, or supplies both stages:

```rust
let shader = MeshShader::fragment(r#"
in vec3 k_world_position;
in vec3 k_normal;
in vec2 k_uv;
out vec4 outColor;
uniform vec4 materialColor;
uniform float amount;

void main() {
    outColor = vec4(materialColor.rgb * amount, materialColor.a);
}
"#);

let group = group_3d()
    .mesh_shader(&shader)
    .mesh_uniform("amount", 0.8)
    .build(s);
let child = sphere()
    .mesh_uniform("amount", 1.0)
    .mesh_shader_bounds(0.25)
    .build(s);
group.add(&child);
```

`group_3d` passes its shader and uniform values to descendants. A child's own
shader wins; compatible local uniforms override inherited values. Mesh sources
omit `#version`. The standard vertex stage exposes `k_world_position`,
`k_normal`, and `k_uv`; missing normals or UVs produce zero values. A custom
vertex stage uses `position` plus available `normal` and `uv_coordinates`
attributes and must actively use `modelMatrix` and `viewProjection`. Optional
engine uniforms are `normalMatrix`, `viewMatrix`, `projectionMatrix`,
`cameraPosition`, `sceneTime`, linear `materialColor`, `hasNormal`, and `hasUv`.

A custom mesh shader replaces the standard material shader, so it receives no
automatic PBR lighting or material textures. Object transforms, depth state,
outline, and transparency selected by material opacity are preserved.
Application-defined `Draw3D` objects must draw through
`RenderContext3D::render_material` to use mesh shaders. GPU vertex deformation
does not update CPU geometry: selection and CPU-side culling keep the original
bounds, expanded conservatively by `.mesh_shader_bounds(...)`. A `group_3d`
remains a mesh hierarchy; canvas-level image shaders are the post-processing
path for flattened 3D output.

Animating a group uniform updates every descendant that inherits it. To animate
one descendant independently, declare a compatible local `.mesh_uniform(...)`
on that descendant's builder first; handler calls never create bindings or
change shader definitions.

For picture-in-picture, texture projection, or other off-screen work, build a
canvas with its own resolution and register it with `Scene::add_canvas_2d` or
`Scene::add_canvas_3d`. Both projection types accept either canvas type:
`projection_2d` displays the canvas in the 2D world, while `projection_3d`
displays it as an unlit plane in the 3D world.

```rust
let canvas = canvas_3d()
    .resolution((640, 360))
    .camera_position(vec3(0.0, 1.0, 4.0))
    .build(s);
s.add_canvas_3d(&canvas);

let projection = projection_2d()
    .source(&canvas)
    .round(16)
    .stroke(Color::WHITE)
    .stroke_width(4.0)
    .build(s);
s.world_2d().add(&projection);
```

`Projection2D` uses a transparent fill by default, so its source remains
visible and a stroke can be drawn over it. Rounded rectangles accept one, two,
three, or four values (`round(10)`, `round([10, 30])`, and so on).

## LaTeX formulas

`Latex2D` uses the native RaTeX layout engine and embedded KaTeX fonts.
Formula geometry is cached and drawn as Skia vector paths. Size, fill, stroke,
transform, and creation effects work like other scene objects.

```rust
let formula = latex_2d()
    .text(r"\frac{1}{2}")
    .size(64.0)
    .build(s);
s.world_2d().add(&formula);
creation().play(&formula);
formula.morph(r"\sqrt{2}").duration(2.0).play();
```

`morph` keeps the same object and changes its source when the tween completes,
including when seeking backward. Use math source without dollar delimiters.
Rendering uses display style; full LaTeX documents and packages are unsupported.
Invalid or unsupported source panics when its geometry is first requested.

## Particle transforms

`morph().play(&from, &to)` replaces an object through particle silhouettes.
It first turns the source into a silhouette, interpolates particle positions and
colors, and resolves the destination into its complete appearance.
Every `spatial = "2d"` object participates automatically: `on_draw` renders only
its normal appearance, while `Draw2D::visual_bounds` reports the local ink bounds.
The default bounds are centered on `box_size`; override them for displaced drawing
or outlines that extend beyond that box.

```rust
let source = circle().radius(80.0).fill(Color::RED).build(s);
let target = text_2d()
    .text("Kinematic")
    .position(vec2(240.0, 0.0))
    .fill(Color::BLUE)
    .build(s);
s.world_2d().add(&source);

morph()
    .duration(2.5)
    .easing(Easing::InOutCubic)
    .fade_from(false)
    .play(&source, &target);

target.position_y(120.0).play();
```

`fade_from` defaults to `true`. Set it to `false` to keep the source visible
while the destination appears. The source must be attached, and the destination
may be unattached or attached to the same parent. The destination keeps its own
local transform. Both objects must contain visible content.
Shapes, text, groups, and custom drawable objects are sampled through their draw
callbacks. Appearances are captured when scheduled; later edits do not change the
captured particle cloud. Masks are limited to 2048 pixels per dimension.

Particle paths are deterministic when seeking, and the original object reappears
when seeking before the effect.
The effect type is `Morph`, distinct from the spatial `Transform`
component and the `Creation`/`Uncreation` effects.

## Scene tree

Every scene owns built-in `World 2D` and `World 3D` canvases at the project
resolution. Objects are inactive after `build()` and begin their
timeline lifetime when added to a container:

```rust
let circle = circle().build(&mut scene);
let child_group = group_2d().build(&mut scene);

child_group.add(&circle);
scene.world_2d().add(&child_group);
```

The `Node` derive gives an object's generated handler the `add` method.
It also provides typed access to direct children in insertion order:

```rust
let circle = child_group
    .get_child::<Circle>(0)
    .expect("First child must be a Circle.");
let first_entity = child_group
    .get_child_entity(0)
    .expect("First child must exist.");
let child_entities = child_group.children();
```

`get_child` returns `ChildError::NotFound` when the index is absent and
`ChildError::TypeMismatch` when the child is a different object type.
`get_child_entity` returns one untyped entity id, while `children` returns
all direct child ids in insertion order.
`Group2D` uses it to organize transformable subtrees, but hierarchy traversal is
not coupled to that concrete type. Container transforms are inherited through
the tree, and container opacity is composited once over its complete subtree at
the destination canvas resolution.

## Canvas cameras

Camera tracks are exposed directly by the canvas handler. Their names use the
`camera_` prefix so they do not conflict with object transform tracks:

```rust
let world = scene.world_2d();
world
    .camera_position(vec2(200.0, 0.0))
    .camera_zoom(2.0)
    .camera_rotation(0.25)
    .duration(1.0)
    .play();
```

`Camera2D` provides `camera_position`, `camera_zoom`, and `camera_rotation`.
`Camera3D` provides `camera_mode`, `camera_position`, `camera_rotation`, `camera_fov`,
`camera_near`, and `camera_far`. The same methods are available on
`canvas_2d()` and `canvas_3d()` builders for initial values:

```rust
let inset = canvas_2d()
    .resolution((640, 360))
    .camera_position(vec2(100.0, 0.0))
    .camera_zoom(1.5)
    .build(&mut scene);
scene.add_canvas_2d(&inset);
```

Camera components belong to their canvas rather than the scene tree. They do
not have builders or handlers of their own, cannot be added or removed, and one
canvas camera never affects another canvas.

`ObjectHandler::remove()` ends the object's lifetime and the lifetimes of all
its descendants. The stored tree remains intact so seeking to an earlier time
restores the subtree. A typed handler exposes its underlying ECS entity through
`ObjectHandler::entity()` when direct identification is needed. Every object
also has a user-facing name. It defaults to its Rust object type, can be set by
the generated builder's `.name(...)` method, and can later be read or changed
through `ObjectHandler::name()` and `ObjectHandler::set_name()`. The scene
root is named `Root`.

## Development

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
```

## License

Kinematic is available under the [MIT License](LICENSE).
