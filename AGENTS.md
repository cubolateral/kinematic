# Kinematic Contributor Guide

## Purpose

Kinematic is a Rust animation-editor library. It uses an ECS scene model, a
timeline-driven animation system, an SDL3/OpenGL renderer (`femtovg`), and an
internal Dear ImGui editor.

This file records stable architectural contracts and contribution workflow. The
source code is the authority for concrete types, available objects, panels, and
other implementation details that can change independently.

## Repository Layout

- `src/app.rs`: SDL, OpenGL, `femtovg`, and ImGui runtime setup.
- `src/core/`: public scene, animation, project, component, and object APIs.
- `src/editor/` and `src/ui/`: crate-internal editor state, preview, timeline,
  and UI.
- `packages/kinematic-macros/`: procedural derives used by the core API.

Keep reusable animation and scene behavior in `core`. Keep windowing, rendering
loop ownership, and ImGui interaction out of the core domain.

## Architectural Contracts

- `SceneBuilder` declares scene contents and schedules animation work through
  `Animator`; it must not own the application loop or editor UI state.
- `Animator` converts declarative `Task` values into entity animation data.
  `Scene::update(time)` evaluates that data; drawing must not mutate animation
  state.
- Start user-facing object construction through `Scene::create::<Object>()`. The
  returned builder's `build` method spawns the object with animation, inspector,
  and inactive `Node` metadata, then returns its typed handler. `Scene::add`
  begins the object's lifetime at the animator's current scheduling time.
  `Scene::destroy` ends that lifetime without despawning the entity. Scene
  evaluation, rendering, and editor views must ignore nodes whose
  `is_activated` value is false.
- Objects are ECS bundles implementing `Object`. The `Object` derive creates the
  `<Object>Builder` returned by `Scene::create` and the typed `<Object>Handler`
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
- `Draw::get_rect` defines an entity's local drawing bounds. `Scene` renders each
  `Draw` into a per-entity offscreen surface, then composites that surface using
  the entity's `Transform` and opacity. `Draw::on_draw` must use local coordinates
  and must not apply the entity transform itself. Drawing callbacks should read
  the ECS world and leave its state unchanged.
- The timeline displays one lifetime rectangle per scene entity across the full
  viewport width. Its preserved track-rendering module is not part of the current
  view and remains available for a later track UI. The viewport supports zoom by
  vertical mouse dragging and horizontal pan by horizontal dragging with the left
  mouse button. The initial drag direction selects the operation, while the right
  mouse button controls the scrubber. It displays adaptive time-grid labels using
  editor-style `1/2/5 × 10ⁿ` intervals.
- Keep timeline viewport and pointer-gesture state in `src/ui`. The editor
  timeline owns playback time and duration. Its `is_controlling` flag is the
  sole UI-related exception because it temporarily suspends playback while the
  scrubber controls the current time.

## API Boundaries

- Treat `src/lib.rs` and `src/prelude.rs` as the public API boundary. Do not make
  editor internals public merely to simplify a local change.
- Prefer extending the public scene/object/animation abstractions over coupling
  downstream code to `hecs::World` internals.
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
6. Keep implementations simple: do not add unnecessary structs, functions, or
   layers of indirection. Introduce an abstraction only when it is genuinely
   needed by the behavior or architecture.

## Maintaining This File

Update this guide only when a stable contract, repository boundary, supported
workflow, or validation command changes. Do not update it for routine additions
or renames of individual components, objects, widgets, easing modes, or other
details discoverable from the source tree.
