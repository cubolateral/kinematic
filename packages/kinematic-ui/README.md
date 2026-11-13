# Kinematic UI

Shared ImGui components and the minimal visual style used by Kinematic. The crate uses `dear-imgui-rs`. It has no
dependency on the application, its editor, scene model, Lua runtime, SDL,
OpenGL, or export loop.

- `theme`: Palette, configurable corners, borderless surfaces, background-derived dark dividers, scale, and font setup. The caller owns
  font assets and serialized preferences.
- `windows`: Fixed docked panels, centered floating windows, panels with embedded footers, tabs, and dialogs.
- `docking`: Single-row panel titles, centered mode controls with a fixed left action, bottom panel selectors with right-aligned metadata, sidebar position grids, and draggable workspace separators. Inactive titles are muted and hover uses the accent.
- `controls`: Buttons, choices, text and color inputs, sliders, menus, and progress.
- `status_bar`: Repository and version information inside panel footers and the seamless Home footer.
- `title_bar`: Window control buttons and their platform layout. The caller supplies available actions and applies window state changes.
- `properties`: Label/value rows, component groups, reset buttons, sections,
  read-only values, and sliders. Shared by Inspector, Sound, and Settings.
- `numbers`: Typed numeric drags, arrow-key editing, and display precision.
- `tree` and `lists`: Hierarchy branches, selection and action targets, file labels,
  suggestions, and shortcut tables. They receive labels and flags, not entities or paths.
- `canvas`: Texture presentation, aspect fitting, zoom/pan, camera masks, margins,
  outlines, and viewport tools. Textures and geometry come from the application.
- `timeline`: Ruler, coordinates, flat blocks, waveforms, and transport. All timeline
  rectangles stay square regardless of the general corner radius.
- `diagnostics`: Severity counters, filtering, and message rows. The caller provides
  locations and receives the ID of a clicked message.
- `icons` and `widgets`: Shared glyphs, text styles, and small layout helpers.

Use these components for ordinary UI. Application panels assemble data and apply component responses. Direct ImGui
access in the application is reserved for lifecycle, docking, input routing, and canvas integration. Custom drawing and visual style belong here.
Add a component when a real view needs it. Do not wrap the entire ImGui API.

```rust,ignore
use kinematic_ui::{controls, properties, windows};

windows::panel(ui, "Options", || {
    properties::field(ui, "Visible", &mut visible, true, |value| {
        controls::checkbox(ui, "##value", value)
    });
});
```

`properties::Edit` separates value changes from completed edits. Inspector edits
can apply immediately. Sound waits for `committed` before saving a slider drag.
Resets commit immediately. Application code owns validation, undo, persistence,
and cancellation.

Run `cargo test -p kinematic-ui` for the component checks.

Backend types are still present at integration boundaries. A future backend
migration will replace those adapters. This crate does not implement a second
backend or a generic rendering framework.
