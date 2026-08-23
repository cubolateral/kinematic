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
- Sequential and parallel animation tasks.
- Built-in easing functions.
- Timeline preview with tracks, keyframes, and inspection data.
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
    let circle = s
        .create::<Circle>()
        .position(vec2(-256.0, 0.0))
        .radius(128.0)
        .fill(Color::RED)
        .build();

    s.add(&circle);

    circle
        .position_x(256.0)
        .duration(1.0)
        .easing(Easing::InOutQuad)
        .play();

    s.wait(1.0);
}

fn main() {
    App::new().run(Project {
        name: "Example",
        resolution: (1280, 720),
        fps: 60,
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
