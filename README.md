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
    App::new().run(Project {
        name: "Example!",
        resolution: (1280, 720),
        fps: 60,
        scenes: vec![example],
    });
}
```

Scene factories in `Project::scenes` run in vector order. Each scene starts as
soon as the previous scene reaches the end of its timeline.

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

## 2D and 3D rendering

Kinematic treats 2D and 3D as equal parts of the same animation model. Skia
renders vector-based `Canvas2D` content, while three-d renders depth-tested
`Canvas3D` content with perspective cameras, meshes, materials, and lighting.
Both dimensions use the same scene tree, typed tracks, timeline, seeking, and
MP4 export workflow.

Every scene includes `World 2D` and `World 3D` canvases at the project
resolution. The 2D world is selected by default. A 3D scene needs a camera,
objects, and a change to the root's `view_2d` track:

```rust
#[scene]
fn scene_3d(s: &mut Scene) {
     s.get_root().view_2d(false).immediate();

    let camera = camera_3d().position(vec3(0.0, 0.0, 3.0)).build(s);
    s.get_world_3d().add(&camera);
    s.get_world_3d().set_camera(&camera);

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
let canvas = canvas_3d().resolution((640, 360)).build(s);
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

## Camera2D

Add a camera to any container to control the rendered view:

```rust
let camera = camera_2d()
    .position(vec2(200.0, 0.0))
    .zoom(2.0)
    .rotation(0.25)
    .build(&mut scene);

scene.get_world_2d().add(&camera);
```

Camera properties belong to `CameraTransform`, separately from the `Transform`
used by drawable objects. A camera can inherit the transform of an ancestor
container. If multiple cameras are active, the last camera in tree order
controls the view. Without an active camera, rendering keeps the identity view.

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
