# UI module guide

`ui::Ui` is the editor UI orchestrator. It owns UI-only state, applies the theme,
creates the workspace, and delegates each panel to its module. Rendering and
interaction details belong in the panel modules, not in `ui/mod.rs`.

## Module map

- `browser.rs`: Internal folder picker with asynchronous listing and navigation.
- `theme.rs`: Appearance state, font loading, colors, scale, and style geometry.
- `workspace.rs`: Default docking layout. It refers to panels through their
  `WINDOW_NAME` constants.
- `inputs.rs`: Action bindings, modifier matching, and shortcut display names.
- `settings.rs`: Separate Project settings and Editor settings windows opened from the menus.
- `code.rs`: Lua buffers, file tabs, focus, and inline LuaLS assistance.
- `code/diagnostics.rs`: Compact diagnostic footer with severity filters and counts.
- `code/shortcuts.rs`: VS Code Linux editing overrides, command chords, and keyboard navigation.
- `code/text.rs`: Word hit testing, occurrence highlights, and selected newline overlays.
- `code/files.rs`: Project file tree, context menu, and file operation dialogs.
- `widgets.rs`: Small drawing and measurement helpers shared by panels.
- `windows.rs`: Shared templates for centered floating windows, modals, and popups.
- `scene_tree.rs`: Active scene hierarchy and entity selection.
- `inspector.rs`: Selected entity inspection.
- `sound.rs`: Independent audio selection and volume/pan editing, including obscured overlaps.
- `export.rs`: Export menu, screenshots and progress.
- `preview/`: Canvas preview, hit testing, and fullscreen presentation.
- `timeline/`: Timeline composition and its independent responsibilities.

The Timeline is split as follows:

- `state.rs`: Persistent Timeline UI state, temporary clip drags and waveform cache.
- `audio.rs`: Collapsible audio track, clip mouse capture, project file drops and editing context menus.
- `waveform.rs`: Bounded peak summaries shared by clips using the same samples.
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
- Keep editor types internal to the binary.
- Do not let preview overlays enter exported frames.

## Common changes

To add a panel:

1. Create its module with a `WINDOW_NAME` constant and a `draw` function.
2. Store panel state beside the other fields in `ui::Ui`, if needed.
3. Call it from `Ui::draw`.
4. Add it to `workspace::apply_default_layout`.

Use `windows::window` for floating windows, `windows::modal` for dialogs, and
`windows::sized_modal` when a dialog needs a specific size. These templates own
initial positioning and sizing; callers supply content and actions. Keep this
module independent of editor and project types so it can move to `kinematic-ui`.
Context menus and inline code assistance keep their contextual anchors.

To change Timeline behavior, update the module that owns that responsibility.
Pure calculations should remain independent of Dear ImGui and have unit tests.

Run `cargo fmt --check`, `cargo check --workspace`, and
`cargo test --workspace` after UI changes.

## Code editor shortcuts

The keymap follows VS Code on Linux for the editor features available here.
Editor Settings → Shortcuts lists both application commands and the CTE editing
bindings. `Ctrl+P` searches project files; arrows choose a result and Enter opens
it. `Ctrl+G` accepts a one-based `line:column`. Closing an editor uses the existing
unsaved-changes dialog. Formatting uses LuaLS without saving the file.

SDL key events normalize punctuation by its layout symbol before reaching
ImGui, so ABNT2 slash and bracket keys share the same shortcuts as US keyboards.
Register every alternative key in routed bindings, including keypad Enter.

CTE owns basic editing, undo, comments, indentation, moving lines, and growing
selections. Register overrides before rendering CTE, through ImGui routing, so
its conflicting defaults do not execute as well. Keep text-field and
documentation focus separate from source editing.

The word at the caret and its whole-word matches receive a subtle highlight.
Selected newlines remain visible across empty lines. Backgrounds use geometry
captured from CTE's child window and render in its transparent parent, behind
the text, with balanced line spacing. Mouse documentation uses the glyph under
the pointer and the child viewport bounds, including after scrolling. Caret
word navigation may look to the left. Drag selection keeps the text cursor
until the mouse button is released, even outside the editor.

The current CTE API cannot create arbitrary additional carets. Multi-cursor
selection uses `Ctrl+D` and `Ctrl+Shift+L`; custom range edits (word deletion,
line duplication/insertion, cutting whole lines, and explicit comment chords) collapse to one caret.
Vertical caret insertion, column selection, workspace search, rename/refactoring,
and a command palette are not implemented.

The audio integration checks need FFmpeg and SDL/OpenGL. On a headless system,
run them sequentially because SDL initialization uses process-wide state:

```sh
SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy cargo test --workspace audio -- --include-ignored --test-threads=1
```
