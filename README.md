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
- First-class 2D and 3D canvases rendered with Skia and three-d, respectively.
- User-facing object names with type-based defaults.
- Hierarchical scene trees with reusable 2D and 3D containers and inherited transforms.
- Animatable orthographic 2D and perspective 3D cameras.
- Built-in and custom `Draw3D` meshes with reusable geometry caches, materials, and lighting.
- Bidirectional projection of 2D and 3D canvases.
- Sequential and parallel animation tasks.
- Named, editable event waits persisted per scene.
- Reactive signals that run after tracks and can temporarily override properties.
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

    s.get_world_2d().add(&circle);

    circle
        .position_x(256.0)
        .fill(Color::BLUE)
        .play();

    s.wait(1.0);
}

fn main() {
    App::new().project("Example", vec![example]).run();
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
    s.get_world_2d().add(&object);

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

For continuous 3D spinning, repeat `object.rotate_y(TAU)` with linear easing.
The axis-angle path preserves direction and full turns; `object.rotation(q)`
interpolates orientations along the shortest quaternion path. Match the cycle's
end and start for a seamless loop; repetition does not automatically close it.

## Signals

Use `signal` to run logic after the scene's tracks have been evaluated. The
callback receives a fresh clone of the object handler on every evaluation, and
direct `set_*` methods temporarily override track values until the next update:

```rust
#[scene]
fn follow_camera(s: &mut Scene) {
    let circle = circle().build(s);
    s.get_world_2d().add(&circle);

    s.get_world_2d().signal(move |handler| {
        handler.set_camera_position(circle.get_global_position());
    });
}
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

canvas.signal(move |handler| {
    handler.set_camera_position(circle_for_signal.get_global_position());
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
    s.get_root().view_2d(false).immediate();

    let group = group_3d().build(s);
    s.get_world_3d().add(&group);

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
in the same scene tree and reuse GPU geometry across objects and frames:

```rust
use kinematic::{hecs, prelude::*, three_d};

#[derive(Clone)]
struct CustomShape;

#[derive(Object, hecs::Bundle)]
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
                get_box: |_, _| Vector3::ONE,
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
s.get_world_2d().add(&projection);
```

`Projection2D` uses a transparent fill by default, so its source remains
visible and a stroke can be drawn over it. Rounded rectangles accept one, two,
three, or four values (`round(10)`, `round([10, 30])`, and so on).

## LaTeX formulas

`Latex` uses the native RaTeX layout engine and embedded KaTeX fonts.
Formula geometry is cached and drawn as Skia vector paths. Size, fill, stroke,
transform, and creation effects work like other scene objects.

```rust
let formula = latex()
    .text(r"\frac{1}{2}")
    .size(64.0)
    .build(s);
s.get_world_2d().add(&formula);
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

```rust
let source = circle().radius(80.0).fill(Color::RED).build(s);
let target = text()
    .text("Kinematic")
    .position(vec2(240.0, 0.0))
    .fill(Color::BLUE)
    .build(s);
s.get_world_2d().add(&source);

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
scene.get_world_2d().add(&child_group);
```

The `Container` derive gives an object's generated handler the `add` method.
`Group2D` uses it to organize transformable subtrees, but hierarchy traversal is
not coupled to that concrete type. Container transforms are inherited through
the tree, and container opacity is composited once over its complete subtree at
the destination canvas resolution.

## Canvas cameras

Camera tracks are exposed directly by the canvas handler. Their names use the
`camera_` prefix so they do not conflict with object transform tracks:

```rust
let world = scene.get_world_2d();
world
    .camera_position(vec2(200.0, 0.0))
    .camera_zoom(2.0)
    .camera_rotation(0.25)
    .duration(1.0)
    .play();
```

`Camera2D` provides `camera_position`, `camera_zoom`, and `camera_rotation`.
`Camera3D` provides `camera_position`, `camera_rotation`, `camera_fov`,
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
`ObjectHandler::get_id()` when direct identification is needed. Every object
also has a user-facing name. It defaults to its Rust object type, can be set by
the generated builder's `.name(...)` method, and can later be read or changed
through `ObjectHandler::get_name()` and `ObjectHandler::set_name()`. The scene
root is named `Root`.

## Development

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
```

## License

Kinematic is available under the [MIT License](LICENSE).
