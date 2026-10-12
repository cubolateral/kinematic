# Lua validation

## Executable migration validation

Validated on October 12, 2026:

- `cargo fmt --check` and `cargo check --workspace` passed.
- `cargo test --workspace` passed outside the sandbox: 300 binary tests and
  three macro tests; five optional tests were ignored by the default suite.
  Inside the sandbox, only the external LuaLS initialization timed out.
- The two optional Lua editor graphics tests passed with
  `SDL_VIDEODRIVER=offscreen`, covering simulation startup/seeking, reload,
  selection, Inspector persistence, project settings, project switching and
  frozen-revision MP4 export verified with FFprobe.
- `cargo run --release -- --check examples/lua/simulation` validated both scenes
  (three and four seconds).
- The release executable opened an empty workspace, opened a temporary copy
  of the existing simulation project, and restored it on a subsequent launch
  without arguments. Its Lua source files remained byte-identical. These
  short offscreen launches were terminated after checking startup; graceful
  desktop interaction was not manually exercised.
- Cargo metadata contains only the Kinematic binary and the internal
  procedural macro target, with publication disabled for both packages.

Manual desktop interaction, Windows/macOS, the separate renderer graphics test,
manual LaTeX review and performance benchmarks were not rerun for this migration.
The remaining sections record earlier validation, not new migration results.

## Earlier Lua integration validation

Validated on Linux x86_64 (Arch kernel 7.1.8), Rust 1.94.1, the existing
SDL/OpenGL desktop session, and FFmpeg 9.0.1. Lua 5.4 is embedded. The installed
Lua Language Server was exercised as a real child process.

## Automated checks

- `cargo fmt --check`.
- `cargo check --workspace --all-targets`, including the editor and internal runtime tests.
- `cargo test --workspace`: 301 passing tests, no failures. The default suite
  excludes two GPU tests, one manual LaTeX layout review and one illustrative
  event doctest.
- `cargo test --bin kinematic graphics -- --ignored --nocapture --test-threads=1`:
  both GPU tests pass in the desktop session.
- `cargo build --bin kinematic`.

The Lua tests cover native and dynamic tracks, hierarchy, snapshots, full-turn
quaternion tweens, identities, failed reloads, removal of shader components,
whole-frame rollback, deterministic random, Game of Life, manual updates,
mutations, checkpoints, simulation dependencies, selective replay and persistent
base overrides. Callback fingerprints ignore formatting and comments while
preserving nested function logic. Simulation libraries cannot hide mutable
state captured during construction.

Document tests exercise atomic saves, external conflicts and project-root
boundaries. LuaLS tests exercise native property completion, module definition
navigation with Unicode, hover and signature help. Missing LuaLS is explicitly
reported by its optional integration test; it was available in this run.

## Real graphics and editor

The editor test creates two projects in separate directories using one App and
ImGui context. It renders the CTE widget, applies unsaved code, preserves entity
selection, rejects invalid code, saves documents, changes resolution, reorders
scenes and releases project resources before switching. During an FFmpeg export,
a newer buffer revision remains pending and every exported frame uses the active
revision. `ffprobe` verifies six frames at the expected 96 × 64 size.

The native graphics test checks pixel colors, orientation, transparency,
2D/3D canvas projections, shapes, shaders and FFmpeg output using actual GL/Skia
resources. It also verifies reuse of the source canvas texture.

The editor smoke test recorded Lua preparation at 3.31–9.50 ms, replay at
0.00–0.07 ms, application at 0.19–2.12 ms, and GPU validation at 0.25–0.84 ms.
These measurements describe its small fixtures, not a large-project benchmark.
The Console reports these phases separately from geometry uploads and rendering.

## Complete example exports

All examples passed `kinematic --check` and were exported with `--render` from
copies in temporary project roots. FFprobe confirmed:

| Project | Resolution | FPS | Frames | Duration |
| --- | --- | --- | --- | --- |
| `examples/lua/2d` | 1280 × 720 | 60 | 750 | 12.5 s |
| `examples/lua/3d` | 1280 × 720 | 60 | 210 | 3.5 s |
| `examples/lua/simulation` | 640 × 360 | 30 | 210 | 7 s |

Extracted frames were inspected for the displaced custom 2D path, the custom
3D mesh with animated shader, canvas composition and Game of Life. Output files
and test caches were kept outside the repository.

## Validation limits

Keyboard/clipboard, IME, drag interactions and each individual CTE command were
not manually exercised. The graphics suite covers internal Rust rendering. The manual LaTeX
layout review remains excluded. Windows and macOS were not tested.

The callback fingerprint parser follows the vendored Lua 5.4 bytecode format;
changing Lua's major version requires updating it and its regression test.
Lua deadlines cannot interrupt an already-running native or GPU operation.
