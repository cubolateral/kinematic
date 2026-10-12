# Kinematic

Kinematic is an animation editor for projects and scenes written in Lua 5.4.
Rust implements the editor, ECS, animation scheduler, simulation and 2D/3D
rendering internally. Kinematic is distributed as an executable, with no Rust
library or scene API.

## Install and run

Build from this repository with a Rust toolchain supporting edition 2024,
CMake and a native C/C++ compiler. SDL3 and Lua are built with the application;
you do not need a separate Lua installation. The editor requires a desktop
session supporting OpenGL 3.3. Install FFmpeg on `PATH` for MP4 export.

```sh
cargo run --release
```

The editor remembers the current project's directory and reopens it on the
next launch. Without a saved project, it opens an empty workspace with the
project creation/opening screen. Closing a project clears the saved current
project. If the directory becomes unavailable, the editor reports the error
and lets you open another project. Project files are never deleted by this flow.

Use **Projects → Open / Create Project** to choose a directory. A new project
contains an editable Lua example, an `assets/` directory and `kinematic.ron`.
Change the scene in Code; the preview updates after a short pause. Save persists
the source files. Closing unsaved buffers or exiting offers Save, Discard and
Cancel.

You can also install the executable with `cargo install --path .` and run
`kinematic`, or open a project directly:

```sh
cargo run --release -- examples/lua/2d
cargo run --release -- --new /tmp/my-animation "My animation"
cargo run --release -- --new /tmp/my-3d-animation "My 3D animation" --3d
cargo run --release -- --check examples/lua/simulation
cargo run --release -- --render examples/lua/3d
```

`--check` constructs and evaluates scenes without opening a window. `--render`
uses the application renderer and exports MP4 to `output/<project name>.mp4`,
adding a numeric suffix when an export already exists.

## Write a scene

```lua
local k = require("kinematic")

return k.scene("Intro", function(s)
    local ball = s:circle {
        id = "ball",
        radius = 24,
        position = k.vec2(-100, 0),
        fill = k.color("#60A5FA"),
    }
    s:world_2d():add(ball)
    s:play(ball.position:to(k.vec2(100, 0), 2, k.ease.linear))
    s:event("Pause")
    s:wait(1)
end)
```

The manifest lists scenes in playback order and stores resolution, FPS and
margins. Configuration defaults to 1280 × 720 at 60 FPS. Edit these values in
the project settings. The existing project format remains unchanged.

Lua supports 2D and 3D objects, hierarchical canvases and projections, property
tweens, sequential/parallel scheduling, loops, snapshots, reactive signals,
deterministic random values, custom drawing, shaders and seekable simulations.
Scene Tree, Timeline and Preview share selection. Inspector edits for explicit
object IDs persist separately from scripts. Code diagnostics and failed reloads
preserve the last valid preview. Export excludes editor overlays.

See [Lua projects and API](docs/lua.md) for the full workflow, scene API,
custom objects, shader contracts, persistence and optional Lua Language Server.
Runnable projects are in [examples/lua](examples/lua): 2D, 3D and simulation,
including Game of Life.

## Storage

Project sources, assets, `kinematic.ron`, runtime metadata under `kinematic/`
and exports under `output/` belong to the project directory. Global settings,
window layout and the current project path stay under `.kinematic` in the
platform's user data directory:

- Linux: `$XDG_DATA_HOME`, or `~/.local/share` when unset.
- macOS: `~/Library/Application Support`.
- Windows: `%APPDATA%`.

## Development

The Rust modules and procedural macros are implementation details. To validate
changes:

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
```

See [validation](docs/lua-validation.md) and
[performance measurements](docs/lua-performance.md) for optional graphics tests
and benchmarks. They do not require publishing a Rust library.

## License

MIT.
