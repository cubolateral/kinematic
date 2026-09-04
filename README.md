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
    let circle = Circle::builder()
        .radius(128.0)
        .position(vec2(-256.0, 0.0))
        .fill(Color::RED)
        .build(s);

    s.get_root().add(&circle);

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

## Scene tree

Every scene owns an internal root container, available through
`Scene::get_root()`. Objects are inactive after `build()` and begin their
timeline lifetime when added to a container:

```rust
let circle = Circle::builder().build(&mut scene);
let child_group = Group::builder().build(&mut scene);

child_group.add(&circle);
scene.get_root().add(&child_group);
```

The `Container` derive gives an object's generated handler the `add` method.
`Group` uses it to organize transformable subtrees, but hierarchy traversal is
not coupled to that concrete type. Container transforms are inherited through
the tree, and container opacity is composited once over its complete subtree at
the destination canvas resolution.

## Camera

Add a camera to any container to control the rendered view:

```rust
let camera = Camera::builder()
    .position(vec2(200.0, 0.0))
    .zoom(2.0)
    .rotation(0.25)
    .build(&mut scene);

scene.get_root().add(&camera);
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
