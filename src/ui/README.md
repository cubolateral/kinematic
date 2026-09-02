# UI module guide

`ui::Ui` is the editor UI orchestrator. It owns UI-only state, applies the theme,
creates the workspace, and delegates each panel to its module. Rendering and
interaction details belong in the panel modules, not in `ui/mod.rs`.

## Module map

- `theme.rs`: Appearance state, font loading, colors, scale, and style geometry.
- `workspace.rs`: Default docking layout. It refers to panels through their
  `WINDOW_NAME` constants.
- `settings.rs`: Configuration panel for `theme::Appearance`.
- `widgets.rs`: Small drawing and measurement helpers shared by panels.
- `scene_tree.rs`: Active scene hierarchy and entity selection.
- `inspector.rs`: Read-only inspection of the selected entity.
- `export.rs`: Export panel and its local UI state.
- `preview/`: Canvas preview, hit testing, and fullscreen presentation.
- `timeline/`: Timeline composition and its independent responsibilities.

The Timeline is split as follows:

- `state.rs`: Persistent Timeline UI state.
- `layout.rs`: Coordinate conversion and calculated panel geometry.
- `metrics.rs`: Timeline-specific dimensions.
- `controls.rs`: Playback shortcuts, transport, and fullscreen controls.
- `interaction.rs`: Mouse gesture state transitions.
- `ruler.rs`: Grid, scrubber, playhead, and time labels.
- `objects.rs`: Object lifetimes, hierarchy rows, and selection.
- `tracks.rs`: Animation tracks, segments, keyframes, and tooltips.

## Boundaries

- Keep project, scene, and animation behavior in `core` and `editor`.
- Keep persistent interaction state in the panel's `State` type.
- Access shared selection and playback through `Editor`.
- Keep drawing helpers stateless and place them in `widgets.rs` only when more
  than one panel uses them.
- Do not move editor-only types into the public API.
- Do not let preview overlays enter exported frames.

## Common changes

To add a panel:

1. Create its module with a `WINDOW_NAME` constant and a `draw` function.
2. Store panel state beside the other fields in `ui::Ui`, if needed.
3. Call it from `Ui::draw`.
4. Add it to `workspace::apply_default_layout`.

To change Timeline behavior, update the module that owns that responsibility.
Pure calculations should remain independent of Dear ImGui and have unit tests.

Run `cargo fmt --check`, `cargo check --workspace`, and
`cargo test --workspace` after UI changes.
