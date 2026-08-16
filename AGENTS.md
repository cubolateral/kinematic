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
- Spawn user-facing scene objects through `Scene::create`. It attaches the
  animation and inspector metadata required by the runtime and editor.
- Objects are ECS bundles implementing `Object`. The `Object` derive creates the
  typed handle returned by `Scene::create`; fields marked `#[trackable]` expose
  trackable component handles through it.
- `Trackable` components expose fields marked `#[track]`. Adding a newly
  trackable value type requires coordinated support in `TrackValue`, its
  interpolation/display behavior, and the `Trackable` derive macro.
- Tracks assume keyframes are appended in non-decreasing timeline order. Preserve
  that invariant when changing task compilation or keyframe insertion.
- `Node` callbacks render entity state only. Rendering code should read the ECS
  world and leave its state unchanged.

## API Boundaries

- Treat `src/lib.rs` and `src/prelude.rs` as the public API boundary. Do not make
  editor internals public merely to simplify a local change.
- Prefer extending the public scene/object/animation abstractions over coupling
  downstream code to `hecs::World` internals.
- Preserve the separation between project definition, animation evaluation,
  scene rendering, and editor controls unless a change intentionally revises the
  architecture.
- Do not use absolute Rust paths with the `::` prefix. Prefer imported names or
  crate-relative paths instead.

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

## Maintaining This File

Update this guide only when a stable contract, repository boundary, supported
workflow, or validation command changes. Do not update it for routine additions
or renames of individual components, objects, widgets, easing modes, or other
details discoverable from the source tree.
