# Kinematic Contributor Guide

## Purpose

Kinematic is a Rust animation-editor library. It uses an ECS scene model, a
timeline-driven animation system, an SDL3/OpenGL runtime, Skia rendering, and an
internal Dear ImGui editor.

This file records stable architectural contracts and contribution workflow. The
source code is the authority for concrete types, available objects, panels, and
other implementation details that can change independently.

## Repository Layout

- `src/app.rs`: SDL, OpenGL, and ImGui runtime setup.
- `src/core/`: public scene, animation, effects, project, component, and object APIs.
- `src/editor/` and `src/ui/`: crate-internal editor state, preview, timeline,
  and UI.
- `src/renderer/`: crate-internal FFmpeg MP4 export pipeline. It reads frames
  from the editor's existing OpenGL preview framebuffer.
- `packages/kinematic-macros/`: procedural derives used by the core API.

Keep reusable animation and scene behavior in `core`. Keep windowing, rendering
loop ownership, and ImGui interaction out of the core domain.

Reusable animation Effects live in `src/core/effects/`. The `Effect` trait is
the public contract for builder-style animations, while individual effect
implementations such as `FadeIn` and `FadeOut` remain in their own modules.

## Architectural Contracts

- `SceneBuilder` declares scene contents and schedules animation work through
  `Scene`; it must not own the application loop or editor UI state.
- The internal `Animator` converts declarative `Task` values into entity
  animation data.
  `Scene::update(time)` evaluates that data; drawing must not mutate animation
  state.
- Start user-facing object construction through `Object::builder()`. The
  returned builder's `build(&mut scene)` method spawns the object with animation, inspector,
  `Name`, and inactive `Node` metadata, then returns its typed handler. Names
  default to the Rust object type, remain readable and mutable through the typed
  handler, and the scene root is named `Root`. Every scene owns a root `Group`
  exposed by `Scene::get_root`. `Group::add` attaches a handler's
  entity and begins its entire subtree's lifetime at the animator's current
  scheduling time. `Group::remove` ends that subtree's lifetime without
  despawning entities or removing their stored tree edges, so seeking to an
  earlier time restores them. Scene evaluation, rendering, and editor views must
  ignore nodes whose `is_activated` value is false.
- Objects are ECS bundles implementing `Object`. The `Object` derive creates the
  `<Object>Builder` returned by `Object::builder` and the typed `<Object>Handler`
  returned by the builder's `build` method. Fields marked `#[trackable]` expose
  their values directly on both types. The
  public API must not require component-level handlers or component-name access
  such as `object.transform().position`; use the flat `object.position` form.
- `Trackable` components expose fields marked `#[track]`. Adding a newly
  trackable value type requires coordinated support in `TrackValue`,
  `TrackValueType`, and its interpolation/display behavior. The `Trackable`
  derive macro accepts every type that implements `TrackValueType`.
- In `Trackable` structs, place fields marked `#[track]` before all untracked
  fields, with one blank line separating the tracked and untracked field groups.
- Tracks assume keyframes are appended in non-decreasing timeline order. Preserve
  that invariant when changing task compilation or keyframe insertion.
- `Scene` starts drawing exclusively from its root `Group`. Groups store ordered
  `Vec<hecs::Entity>` child lists and recursively invoke each active child's
  `Draw` callback using the child's `Transform`, so a child must belong to at
  most one group. Group opacity must composite the subtree through a Skia layer
  at the destination resolution. Non-group `Draw::on_draw` callbacks must use
  local coordinates, apply the entity's opacity to their paints, and must not
  apply the entity transform itself. Drawing callbacks should read the ECS world
  and leave its state unchanged.
- Timeline rows follow the root Group's pre-order hierarchy. Object rows are
  collapsed by default; a double-click toggles track rows directly below the
  object, with track lines constrained to its lifetime. The viewport supports
  zoom by vertical mouse dragging and horizontal pan by horizontal dragging
  with the left mouse button. Selection and object toggles are committed only
  when the left button is released over the pressed target without crossing the
  drag threshold. The initial drag direction selects the operation, while the
  right mouse button controls the scrubber. It displays adaptive time-grid
  labels using editor-style `1/2/5 × 10ⁿ` intervals.
- Keep timeline viewport and pointer-gesture state in `src/ui`. The editor
  timeline owns playback time and duration. Its `is_controlling` flag is the
  sole UI-related exception because it temporarily suspends playback while the
  scrubber controls the current time.
- Keep entity selection as shared editor state. The Scene Tree, Timeline, and
  Preview may update the selected entity, while the Inspector only reads it and
  displays the selected object's properties. The Preview outlines the selected
  active entity using its rendered transform and local bounds; this editor
  overlay must not enter exported frames. Preview hit-testing follows reverse
  drawing order so the frontmost overlapping object is selected, then falls
  back to a containing Group. Scene Tree, Timeline, and Preview clear selection
  when clicking outside every object. Their interaction remains disabled
  throughout export.
- The renderer is crate-internal and must reuse the application's existing SDL,
  OpenGL, Skia, and preview framebuffer resources. It must not create a second
  window or graphics context. Export advances the scene by exactly `1 / fps`
  per frame, sends raw RGBA frames to FFmpeg, and writes MP4 files to
  `output/<project name>.mp4`. FFmpeg must be available in `PATH`.

## API Boundaries

- Treat `src/lib.rs` and `src/prelude.rs` as the public API boundary. Do not make
  editor internals public merely to simplify a local change.
- Keep wildcard reexports local to the module that owns the items. Parent
  modules must expose child directories with `pub mod module;` only; do not
  reexport child-module contents with `pub use module::*`. Public convenience
  reexports for submodules belong in `src/prelude.rs`.
- Prefer extending the public scene/object/animation abstractions over coupling
  downstream code to `hecs::World` internals.
- Public Effects belong under `core/effects` and should compose the existing
  `ObjectHandler` property animation API instead of requiring per-object Effect
  implementations.
- Preserve the separation between project definition, animation evaluation,
  scene rendering, and editor controls unless a change intentionally revises the
  architecture.
- Do not use absolute Rust paths with the `::` prefix. For items defined by
  this crate, always add a `use crate::...` import and refer to the imported
  name (`Tween`, `SceneWorld`, and so on) instead of spelling paths such as
  `crate::core::Tween` throughout the implementation. This rule also applies
  to generated code whenever the macro can emit the corresponding import.
- In `src/`, reserve `use` declarations for project modules. Refer to ordinary
  external dependency items with qualified paths. Derive macro imports such as
  `kinematic_macros` and extension traits required for method resolution such as
  `glow::HasContext` are exceptions.

## Change Workflow

1. Inspect the relevant module and its callers before changing a core contract or
   macro-generated API.
2. Keep procedural macro output and the traits/types it implements in sync.
3. For Rust changes, run `cargo fmt --check` and `cargo check --workspace`.
   Run focused tests when they exist for the behavior being changed.
4. Avoid introducing new public API, dependencies, or `unsafe` code without a
   concrete architectural need.
5. Start string literals and comments with an uppercase letter, and end them
   with a period or exclamation mark.
6. Use moderate vertical spacing in Rust code, including closures and
   callbacks. Keep closely related variables and statements together, and add
   one blank line between distinct stages of a function or groups of variables
   with different purposes. Avoid blank lines between every individual
   statement.
7. Keep implementations simple: do not add unnecessary structs, functions, or
   layers of indirection. Introduce an abstraction only when it is genuinely
   needed by the behavior or architecture.

## Maintaining This File

Update this guide only when a stable contract, repository boundary, supported
workflow, or validation command changes. Do not update it for routine additions
or renames of individual components, objects, widgets, easing modes, or other
details discoverable from the source tree.
