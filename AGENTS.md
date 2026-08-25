# Kinematic — Contributor Guide

Kinematic is a Rust animation editor and library. The source code is the authority for concrete APIs, panels, object types, and implementation details. Keep this file limited to stable rules so it should rarely need updates.

## Architecture

- src/core/ contains the reusable public scene, object, animation, effect, project, and component model.
- src/editor/ and src/ui/ contain crate-private editor state and Dear ImGui views.
- src/app.rs owns SDL, OpenGL, ImGui, and the application loop.
- src/renderer/ owns MP4 export and reuses the application's existing graphics resources.
- packages/kinematic-macros/ contains the procedural macros used by the public API.
- Keep windowing, rendering-loop ownership, and editor interaction out of core.

## Stable contracts

- SceneBuilder declares scene contents and schedules work through Scene; it does not own the application loop or editor state.
- Animator compiles Task values into animation data. Scene::update(time) evaluates that data. Drawing must not mutate animation state.
- User-facing objects are created with Object::builder().build(&mut scene). Objects have typed handlers, a default type name, and inactive Node metadata until attached to a group.
- Every scene has a named root Group available through Scene::get_root(). Group::add attaches a subtree at the animator's current time. Group::remove ends its lifetime without deleting entities or tree edges, so seeking backward can restore it.
- Inactive nodes must be ignored by scene evaluation, rendering, hit testing, and editor views.
- Objects are ECS bundles implementing Object. The derive macro generates builders and typed handlers. #[trackable] fields are exposed directly on those types; do not introduce component-level handlers for convenience.
- Trackable fields use #[track] and must appear before untracked fields, separated by one blank line. New trackable value types require support in TrackValue, TrackValueType, interpolation, and display behavior.
- Keyframes are appended in non-decreasing timeline order.
- Drawing starts at the root group, follows ordered child lists, uses local coordinates in non-group draw callbacks, applies opacity in paints, and leaves ECS state unchanged. Group opacity composites the complete subtree once.
- Timeline state belongs in src/ui; playback time and duration belong to the editor timeline. Selection is shared by Scene Tree, Timeline, and Preview; the Inspector only reads it. Editor overlays must never enter exported frames.
- Export reuses the existing SDL/OpenGL/Skia/framebuffer resources, advances exactly 1 / fps per frame, streams raw RGBA frames to FFmpeg, and writes output/<project name>.mp4.

## API and style boundaries

- src/lib.rs and src/prelude.rs are the public API boundary. Do not expose editor internals to simplify a local change.
- Keep convenience reexports in prelude.rs; avoid adding public API solely for internal implementation needs.
- For crate items, import with use crate::... and do not use absolute paths beginning with ::. Qualified paths are fine for external dependencies.
- Preserve the separation between project definition, animation evaluation, rendering, and editor controls unless intentionally changing the architecture.
- Keep comments and string literals concise, start them with an uppercase letter, and end them with punctuation.
- Prefer simple implementations. Avoid new layers, dependencies, unsafe code, or public types unless they are needed by the behavior.

## Validation

For Rust changes, run cargo fmt --check, cargo check --workspace, and cargo test --workspace. Inspect callers before changing a core contract or macro-generated API. Keep macro output and its implementing traits/types synchronized.
