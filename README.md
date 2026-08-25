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
- Hierarchical scene trees with nested groups, inherited transforms, and
  composited group opacity.
- Sequential and parallel animation tasks.
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
        .name("Main Circle")
        .radius(128.0)
        .position(vec2(-256.0, 0.0))
        .fill(Color::RED)
        .build(s);
    let group = Group::builder()
        .opacity(1.0)
        .build(s);

    group.add(&circle);
    s.get_root().add(&group);

    circle
        .position_x(256.0)
        .fill(Color::BLUE)
        .duration(1.0)
        .easing(Easing::InOutQuad)
        .play();

    s.wait(1.0);
    group.remove(&circle);
}

fn main() {
    App::new().run(Project {
        name: "Example!",
        resolution: (1280, 720),
        fps: 60,
        scene: example(),
    });
}
```

## Scene tree

Every scene owns a root [`Group`](src/core/objects/group.rs), available through
`Scene::get_root()`. Objects are inactive after `build()` and begin their
timeline lifetime when added to a group:

```rust
let circle = Circle::builder().build(&mut scene);
let child_group = Group::builder().build(&mut scene);

child_group.add(&circle);
scene.get_root().add(&child_group);
```

Groups may contain objects or other groups. Transforms are inherited through
the tree, and group opacity is composited once over the complete subtree at the
destination canvas resolution.

`Group::remove()` ends the selected object's lifetime and the lifetimes of all
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
