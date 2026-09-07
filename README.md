# Kinematic

Kinematic is a Rust animation editor and library built around an ECS scene,
typed animation tracks, and a timeline-based workflow. It uses
[SDL3](https://github.com/libsdl-org/SDL) and OpenGL for the application runtime,
[Skia](https://skia.org/) for drawing, and
[Dear ImGui](https://github.com/ocornut/imgui) for the editor UI, and
[FFmpeg](https://ffmpeg.org/) as its video exporter.

Kinematic is in early development, so its API may change.

## Features

- Typed scene objects and trackable component fields.
- User-facing object names with type-based defaults.
- Hierarchical scene trees with reusable containers and inherited transforms.
- Animatable cameras with position, zoom, and rotation.
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

The built-in 2D world is rendered by default. A 3D scene only needs its camera,
objects, and a change to the root's `view_2d` track:

```rust
#[scene]
fn scene_3d(s: &mut Scene) {
    s.get_root().view_2d(false).immediate();

    let world_2d = s.get_world_2d();
    world_2d.add(&text().build(s));

    let world_3d = s.get_world_3d();
    let camera = camera_3d().build(s);
    world_3d.add(&camera);
    world_3d.set_camera(&camera);

    let screen = projection().source(&world_2d).build(s);
    world_3d.add(&screen);
    world_3d.add(&cuboid().build(s));
}
```

`view_2d` is a discrete boolean track: `true` selects `World 2D` and `false`
selects `World 3D`. Schedule additional immediate changes to alternate between
them during the video.

For picture-in-picture or other off-screen work, build a canvas with its own
resolution and attach it with `Scene::add_canvas_2d` or
`Scene::add_canvas_3d`.

## LaTeX formulas

`Latex` uses the native RaTeX layout engine and embedded KaTeX fonts.
Formula geometry is cached and drawn as Skia vector paths. Size, fill, stroke,
transform, and creation effects work like other scene objects.

```rust
let formula = latex()
    .text(r"\frac{1}{2}".to_owned())
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
    .text("Kinematic".to_owned())
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
