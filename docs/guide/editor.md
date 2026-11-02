# The editor

## Panels and selection

| Area | Use |
| --- | --- |
| Home | Create and open projects, find recent projects, and open settings. |
| Files | Browse, create, rename, duplicate, and open source and asset files. |
| Code | Edit Lua in tabs, with diagnostics, completion, search, and replace. |
| Preview | View the scene, select objects, and navigate with the editor cameras. |
| Scene Tree | Inspect active objects and select them in the hierarchy. |
| Inspector | Read and preview properties for the selected object. |
| Timeline | View scenes, properties, events, playback, and audio clips. |
| Console | Read prints, errors, and reload timings. |
| Sound | Select clips, including overlapping clips, and adjust volume and pan. |

Scene Tree, Timeline, and Preview share object selection. Inspector reads that
selection. Audio selection is separate. Inactive objects do not appear in editor views.
Rearrange panels by docking them. Window positions and layouts are not saved.

## Navigate the preview

Press `F1` to focus the preview, `F2` for 2D editing, and `F3` for 3D editing. Editor
cameras let you inspect a composition without changing the scene's animated camera.
Press `0` to switch the camera view and `R` to reset the navigation transform. In 3D,
use W/A/S/D to move, Shift to move faster, and the directional views listed in **Editor
settings → Shortcuts**. Press `F` for fullscreen and `M` to show or hide margins.

The video camera belongs to the canvas. Change it in code to affect the result.
Selection and navigation overlays do not appear in the MP4.

## Preview properties with Inspector

Pause before changing properties. An Inspector edit applies only at the current time. It
does not change Lua or save to the project, and it disappears when playback resumes, the
timeline moves, or the scene reloads. This also applies to objects with explicit IDs. To
keep a value, update the constructor options or add an animation in code.

## Timeline and events

Use the ruler to seek. The arrow keys move between frames, and Space toggles playback.
Project scene order comes from the manifest. Focus a scene to limit the visible and
audible range without changing its clips. Press `L` to loop playback in the editor.

Use `s:event("Reading")` to add a named pause that you can adjust on the timeline.
Resolve pending source edits before changing events. Event times are saved in the [scene
metadata](projects.md#reloads-and-revisions).

Property channels reflect animations prepared in Lua. For imported sounds, use the
**Audio** area to move, trim, split, or delete clips. See [audio editing](audio.md).

## Code and diagnostics

Completion appears while typing and after `.` or `:`. Press `Ctrl+Space` to request it.
Use the arrow keys to choose a result, Tab or Enter to insert, and Escape to close.
Hover over a symbol for documentation. Select and copy text from the popup. Press `F12`
or `Ctrl+click` to go to a definition. Signatures appear while you enter arguments.

LuaLS uses definitions generated from the same source as the [API
reference](../api/index.md). Logs and definitions are stored in `kinematic/cache/luals`.
Server failures appear as diagnostics but do not prevent basic editing. Replies for
older buffer revisions are discarded.

Select a diagnostic to open its file, line, and column. Console keeps up to 500 lines
and suppresses consecutive identical prints. The status shows pending edits, replay, or
failure. A visible image may still belong to the previous revision.

## Settings and shortcuts

**Project settings** controls the project. **Editor settings** controls appearance and
editing behavior, and lists all shortcuts. Global preferences and recent projects are
stored in `.kinematic` in the user data directory. Fonts and the manifest are stored in
the project. Preview volume and code zoom are restored when you reopen the editor.

The default editor background is RGB `(10, 10, 10)` with full opacity, contrast is `0.5`,
and the shared corner radius is `4 px`. UI surfaces, hover and active states, borders,
and disabled text are derived from the background. The docs site uses this same
calculation, so changing its `BACKGROUND` and `CONTRAST` in
`docs/.vitepress/theme/index.ts` updates all derived colors. Kinematic linearizes each sRGB
background channel (`c / 12.92` when `c <= 0.04045`, otherwise
`((c + 0.055) / 1.055)^2.4`) and computes luminance as
`0.2126 R + 0.7152 G + 0.0722 B`. Luminance above `0.179` selects black foreground;
otherwise it selects white. For each derived color, Kinematic mixes each RGB channel
with that foreground using `background + (foreground - background) * amount * contrast`;
the amount is `0.08` for surfaces, `0.14` for hover, `0.22` for active states, `0.12`
for borders, and `0.50` for disabled text. Alpha stays equal to the background alpha.

The code editor defaults to the One Dark syntax palette, with its background set to the
same RGB `(10, 10, 10)` value. Its syntax colors remain those of One Dark.

| Action | Shortcut |
| --- | --- |
| Save file or project | Ctrl+S. |
| Return to projects | Ctrl+Shift+Q. |
| Exit | Ctrl+Q. |
| Play or pause | Space. |
| Previous or next frame | Left or right arrow. |
| Go to start or end | Shift+left or right arrow. |
| Split audio | Ctrl+B. |
| Delete selected clip | Delete or Backspace. |
| Find or replace in code | Ctrl+F / Ctrl+H. |
| Increase or decrease font size | Ctrl++ / Ctrl+-. |
| Go to definition | F12 or Ctrl+click. |

Editor shortcuts do not capture keys typed into text fields. Find the full list and
navigation bindings under **Shortcuts**, generated from the active action map.
