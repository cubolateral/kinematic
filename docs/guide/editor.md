---
pageClass: guide-page
---

# The editor

Start with a project open. Follow one object from selection to inspection, then change
its source and verify that the new revision was applied. The following tasks use the
same panels you will use while building an animation.

## Select an object

Click an active object in Preview, or its entry in Scene Tree or Timeline. The other
selection views follow it and Inspector shows its properties grouped by component name. Pause at a time when the
object exists. An object outside its lifetime will not be available to select.

Try selecting the circle from [first steps](start.md), then seek beyond a scheduled
removal. Seek back to recover the object and its earlier appearance.

## Find the panel for a task

| Area | Use |
| --- | --- |
| Home | Create and open projects, find recent projects, and open settings. |
| Files | Browse, create, rename, duplicate, and open source and asset files. |
| Preview | View the scene, select objects, and navigate with the editor cameras. |
| Scene Tree | Inspect active objects and select them in the hierarchy. |
| Inspector | Read and preview properties for the selected object. |
| Timeline | View scenes, properties, events, playback, and audio clips. |
| Console | Read prints, errors, and reload timings. |
| Sound | Select clips, including overlapping clips, and adjust volume and pan. |

Scene Tree, Timeline, and Preview share object selection. Inspector reads that
selection. Audio selection is separate. Inactive objects do not appear in editor views.

Click empty space in Files to clear the file selection.
Move sidebar panels with their dock position selector. Sidebar positions and sizes
are saved automatically. Preview remains above the shared Timeline and Console area.

## Navigate the preview

Use the centered **Preview**, **2D**, and **3D** controls above the viewport, or
press `F1`, `F2`, and `F3` to switch modes. Editor
cameras let you inspect a composition without changing the scene's animated camera.
Press `0` to switch the camera view and `R` to reset the navigation transform. In 3D,
use W/A/S/D to move, Shift to move faster, and the directional views listed in **Editor
settings → Shortcuts**. Press `F` for fullscreen and `M` to show or hide margins.

The video camera belongs to the canvas. Change it in code to affect the result.
Selection and navigation overlays do not appear in the MP4.

## Inspect and try a value

Pause before changing properties. An Inspector edit applies only at the current time. It
does not change Lua or save to the project, and it disappears when playback resumes, the
timeline moves, or the scene reloads. This also applies to objects with explicit IDs. To
keep a value, update the constructor options or add an animation in code.

## Timeline and events

Use the ruler to seek. The arrow keys move between frames, and Space toggles playback.
Project scene order comes from the manifest. Focus a scene to limit the visible and
audible range without changing its clips. Press `L` to loop playback in the editor.

Use `k.event("Reading")` to add a named pause that you can adjust on the timeline.
Resolve pending source edits before changing events. Event times are saved in the [scene
metadata](projects.md#reloads-and-revisions).

Property channels reflect animations prepared in Lua. For imported sounds, use the
**Audio** area to move, trim, split, or delete clips. See [audio editing](audio.md).

## Edit and diagnose a revision

Open a file from the **Files** tree to edit it externally. The VS Code and VSCodium
profiles also pass the project folder, leaving window selection to the external editor.
**Editor settings → External editor** offers VS Code, VSCodium, and a custom
graphical editor. The executable may be a full path or a command on PATH; enter it
without surrounding quotes. Relative executable paths containing directories resolve
against the project. On Windows, choose the installed `Code.exe` or `VSCodium.exe`.
No terminal editor or Kinematic editor extension is required.

Project and file arguments are JSON arrays of separate arguments. For VS Code and
VSCodium, the defaults are:

```json
["{project}"]
["{project}", "--goto", "{file}:{line}:{column}"]
```

`{project}` and `{file}` become absolute paths; `{line}` and `{column}` start at 1.
For a custom editor, follow its command-line syntax. Spaces and shell characters in
paths stay inside their argument; commands run without a shell. **Test project** and
**Test file** require an open project; the latter opens a scene at line 7, column 3.
Check the external window to confirm navigation. Launch and exit failures appear in
Console or the settings test result. Opening an editor never waits for it on the UI
thread.

Install the LuaLS integration provided by your external editor. Kinematic generates
`kinematic/cache/luals/kinematic.lua` from the same metadata as the [API
reference](../api/index.md), plus `components.lua` for custom components and objects.
Opening the project refreshes these files and creates `.luarc.json` if neither
`.luarc.json`, `.luarc.jsonc`, nor `.vscode/settings.json` exists. Existing configuration
is preserved. If you already have LuaLS settings, merge the entries from
`kinematic/cache/luals/luarc.json` into them, preserving your own library paths and
other choices. In editor-specific settings, prefix keys with `Lua.`; `.luarc.json`
uses unprefixed keys. If you already use `runtime.plugin`, integrate both plugins
manually rather than replacing it. LuaLS may ask you to trust the generated plugin.
See [LuaLS configuration](https://luals.github.io/wiki/configuration/) and
[plugin settings](https://luals.github.io/wiki/settings/#runtimeplugin).

Completion, hover, definitions, and formatting belong to the external editor. Save
Lua files there to update Kinematic. The existing file watcher also handles atomic
replacement saves and changes to required modules.

For a concrete check, change a circle radius to a string. Read the diagnostic and
notice that the last valid scene stays visible. Restore a number and check that the
pending or failure status clears before exporting.

Select a runtime diagnostic in Console to open its file, line, and column externally. Console keeps up to 500 lines
and suppresses consecutive identical prints. The status shows pending edits, replay, or
failure. A visible image may still belong to the previous revision.

The interface uses borderless surfaces separated by darker gaps derived from the
background and contrast. **Border radius** rounds shared controls, menu and list highlights, windows, and dialogs;
timeline blocks keep square corners.

The Home screen keeps its top bar and footer, with no separators against its content.

## Settings and shortcuts

**Project settings** controls the project. **Editor settings** controls appearance and
editing behavior, including border radius and window spacing, and lists all shortcuts. Global preferences and recent projects are
stored in `.kinematic` in the user data directory. Fonts and the manifest are stored in
the project. Settings windows and dialogs can be moved; only the dock position popup stays fixed. Preview volume and external editor preferences are restored when you reopen the editor.

| Action | Shortcut |
| --- | --- |
| Save project | Ctrl+S. |
| Return to projects | Ctrl+Shift+Q. |
| Exit | Ctrl+Q. |
| Show or hide sidebars | Tab. |
| Play or pause | Space. |
| Previous or next frame | Left or right arrow. |
| Go to start or end | Shift+left or right arrow. |
| Split audio | Ctrl+B. |
| Delete selected clip | Delete or Backspace. |

Files, Scene Tree, and Inspector occupy eight fixed dock positions: two columns,
each with an upper and lower position, on either side of the editor. Use the **...**
button in a panel to choose its position. Panels in the same position share one title row. Click a title to switch panels;
the active title uses normal text, other titles are muted, and hovering uses the accent color.
Resize the sidebars and their occupied divisions by dragging the separators.
Positions and sidebar sizes are restored when you reopen the editor.

**Tab** or the sidebar button at the left of the Preview mode controls hides and
restores both sidebars. Empty positions take
no space. Preview always occupies the center, with Timeline and Console sharing the
bottom dock. Switch panels using the names below the content. Click the active name
to collapse the content and expand Preview; click either name to reopen it. Drag the
separator above the content to resize it. Repository and version information stay
at the right of this same row inside the active Timeline or Console panel. Panels cannot float or be closed.

Editor shortcuts do not capture keys typed into text fields. Find the full list and
navigation bindings under **Shortcuts**, generated from the active action map.
