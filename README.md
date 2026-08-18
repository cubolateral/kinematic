# Kinematic

Kinematic is a Rust animation editor and library built around an ECS scene,
typed animation tracks, and a timeline-based workflow. It uses
[SDL3](https://github.com/libsdl-org/SDL) and OpenGL for the application runtime,
[femtovg](https://github.com/femtovg/femtovg) for drawing, and
[Dear ImGui](https://github.com/ocornut/imgui) for the editor UI.

Kinematic is in early development, so its API may change.

## Features

- Typed scene objects and trackable component fields.
- Sequential and parallel animation tasks.
- Built-in easing functions.
- Timeline preview with tracks, keyframes, and inspection data.
- SDL3/OpenGL rendering with an internal Dear ImGui editor.

## Requirements

- A Rust toolchain with edition 2024 support.
- CMake to build the bundled SDL3 source.
- A native C/C++ toolchain to compile SDL3 and Dear ImGui.
- A desktop environment with OpenGL 3.3 support.

## Example

```rust
use kinematic::prelude::*;

#[scene]
fn example(s: &mut Scene, a: &mut Animator) {
    let circle = s.create(
        CircleBuilder::new()
            .position(vec2(-256.0, 0.0))
            .radius(128.0)
            .fill(Color::RED)
            .build(),
    );

    a.tween(
        circle
            .position
            .x(256.0)
            .duration(1.0)
            .easing(Easing::InOutQuad),
    );

    a.wait(1.0);
    s.destroy(circle);
}

fn main() {
    App::new().run(Project {
        name: "Example",
        resolution: (1280, 720),
        scene: example(),
    });
}
```

## Development

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
```

## License

Kinematic is available under the [MIT License](LICENSE).
