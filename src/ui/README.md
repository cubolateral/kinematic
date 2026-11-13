# UI module guide

`ui::Ui` is the editor UI orchestrator. It owns UI-only state, applies the theme,
creates the workspace, and delegates each panel to its module. Project-specific
interaction belongs in the panel modules. Reusable presentation belongs in
`packages/kinematic-ui`, not in the application panels.

## Module map

- `browser.rs`: Internal folder picker with asynchronous listing and navigation.
- `theme.rs`: Persisted appearance settings and application font assets. Delegates styling to `kinematic_ui::theme`.
- `workspace.rs`: Fixed sidebar positions, resizing, bottom panel selectors, and the center docking layout. It refers to panels through their
  `WINDOW_NAME` constants.
- `inputs.rs`: Action bindings, modifier matching, and shortcut display names.
- `settings.rs`: Separate Project settings and Editor settings windows opened from the menus.
- `title_bar.rs`: Shared window controls, Home settings, and the empty region used for system window dragging. SDL operations stay in `src/app/window.rs`.
- `code.rs`: Project dialogs, Files, Console, and external editor navigation.
- `code/diagnostics.rs`: Converts runtime messages into UI diagnostics and routes clicked locations.
- `code/files.rs`: Project file tree, context menu, and file operation dialogs.
- `scene_tree.rs`: Active scene hierarchy and entity selection.
- `inspector.rs`: Selected entity inspection.
- `inspector/audio.rs`: Independent audio selection and volume/pan editing, including obscured overlaps.
- `export.rs`: Export menu and progress.
- `preview/`: Canvas preview, zoom/pan, playback controls, hit testing, and fullscreen presentation.
- `timeline/`: Timeline composition and its independent responsibilities.

The Home bar shows editor settings. The project bar shows the project, editor,
view, and export menus. Window controls share that bar. SDL handles dragging and edge
resizing, with native decorations retained if hit testing is unavailable.
On Linux, button availability follows the desktop session because SDL does not
expose the compositor's individual window management capabilities.
During native move and resize loops, `src/app/live_redraw.rs` requests UI frames
on SDL expose notifications. It reuses the current scene textures without
advancing the animation evaluator or export frames.

The Timeline is split as follows:

- `state.rs`: Persistent Timeline UI state, temporary clip drags and waveform cache.
- `audio.rs`: Collapsible audio track, clip mouse capture, project file drops and editing context menus.
- `interaction.rs`: Mouse gesture state transitions.
Ruler, layout, metrics, waveform summaries, block painting, and transport are in
`kinematic_ui::timeline`. Preview geometry and overlays are in `kinematic_ui::canvas`.

## Boundaries

- Keep project, scene, and animation behavior in `core` and `editor`.
- Keep persistent interaction state in the panel's `State` type.
- Access shared selection and playback through `Editor`.
- Use `kinematic_ui::{controls, numbers, properties, windows}` for ordinary controls and layouts.
- Keep custom drawing, colors, spacing, and shapes in `kinematic-ui`.
- Keep direct ImGui access only at lifecycle, docking, input-routing, and canvas integration boundaries. Do not create panel-specific copies of shared components.
- Keep editor types internal to the binary.
- Do not let preview overlays enter exported frames.

## Common changes

To add a panel:

1. Create its module with a `WINDOW_NAME` constant and a `draw` function.
2. Store panel state beside the other fields in `ui::Ui`, if needed.
3. Call it from `Ui::draw`.
4. Add its fixed position and dock assignment to `workspace::State`.

Use `kinematic_ui::windows::panel` for docked panels, `windows::window` for
floating windows, `windows::modal` for dialogs, and `windows::sized_modal` when
a dialog needs a specific size. These templates own positioning and sizing.
callers supply content and actions. Keep them independent of editor and project types.
Context menus keep their contextual anchors. Panel groups use `docking::header`
for a single row of titles. Apply requested panel changes before drawing docked
windows. Shared windows inherit borderless surfaces and the configured corner
radius; workspace gaps derive their darker color from the theme background and contrast.

To change Timeline behavior, update the module that owns that responsibility.
Pure calculations should remain independent of Dear ImGui and have unit tests.

Run `cargo fmt --check`, `cargo check --workspace`, and
`cargo test --workspace` after UI changes.

External editor settings live in `settings.rs`. `editor/external.rs` owns process
launching, argument expansion, and generated LuaLS configuration. `Documents` retains
file watching and the saved source snapshots used by scene reload and metadata writes.
The external editor owns text editing, completion, formatting, and unsaved changes.

The audio integration checks need FFmpeg and SDL/OpenGL. On a headless system,
run them sequentially because SDL initialization uses process-wide state:

```sh
SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy cargo test --workspace audio -- --include-ignored --test-threads=1
```
