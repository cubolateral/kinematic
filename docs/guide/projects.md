# Projects and scenes

## Project structure

The project root is the folder above `kinematic/`. Keep scenes in `scenes/`, assets in
`assets/`, and modules wherever they make sense for your project. Exports go in
`output/`. Editor caches and metadata are stored in `kinematic/`.

```text
my-project/
  kinematic/project.ron
  scenes/intro.lua
  assets/font.ttf
  modules/colors.lua
```

## project.ron and project settings

The manifest is the authoritative project configuration. Do not add fields the current
format does not support. This is the current format:

<<< ../../examples/lua/2d/kinematic/project.ron

`version` identifies the manifest format, not the application version. `scenes` lists
the scene files and their order in the video. Each file returns a `function(s)`. The
displayed scene name comes from its file name, while its persistent identity is its
relative path. A module does not become a scene just because it is in the project.

`resolution` sets the width and height in pixels. `fps` sets the frame rate and
simulation step. Dimensions are limited to 8192 and FPS to 1000. `margin` adds
composition guides without cropping the exported image. Use **Project → Project
settings** to set the name, dimensions, FPS, and margins. Press `Ctrl+S` to save.

## Assets, paths, and modules

Fonts, images, GLSL, and audio paths are relative to the project root, including paths
used by modules. Absolute paths, `..`, and symbolic links that leave the root are
rejected. Files are frozen into the applied revision when read. Editing a file creates a
new revision and does not change an export already in progress.

```lua
-- modules/colors.lua.
return {accent = k.color("#38BDF8")}
```

```lua
local colors = require("modules.colors")

return function(s)
    s:world_2d():add(s:circle {fill = colors.accent})
    s:wait(1)
end
```

`require("folder.module")` looks for `folder/module.lua` and `folder/module/init.lua`.
Imports are cached per revision, and circular imports are errors. `k` is available in
modules as a read-only API. Simulations use a [restricted module
environment](simulation.md).

## IDs and object identity

Give important objects an explicit `id`, for example `s:circle {id="ball"}`. IDs are
unique within a scene and preserve identity when declarations are inserted or reordered.
Without an ID, identity depends on declaration order. Changing a built-in type or
dynamic schema may recreate incompatible data.

IDs preserve selection and reconciliation, but they do not make Inspector changes
persistent. Objects missing from a new revision are removed without accumulating tracks
from previous revisions.

## Reloads and revisions

The editor prepares a candidate revision from open buffers, including unsaved text. It
replays any required simulation and validates resources and drawing before applying the
candidate between frames. Errors leave the last valid preview in place. Modules and
resources determine which scenes need to be reloaded. Long replays can be cancelled
between time slices.

Exports use the latest successfully applied revision. If a buffer contains an error, the
code in **Code** may differ from the exported video. Check the editor status before
exporting. Further changes wait until an export finishes or is cancelled.

Editable timeline events are stored separately from Lua in `kinematic/scenes/`. For
example, `scenes/intro.lua` corresponds to `kinematic/scenes/scenes/intro.ron`. Audio
clips are stored in the manifest. Caches can be regenerated. Keep event metadata when
sharing edits that include it.

## Files and conflicts

Use **Files** to create, open, and rename files or folders. Renaming does not update
`require` statements or resource paths. When closing an unsaved file, choose Save,
Discard, or Cancel. Saves use atomic replacement and check the on-disk version. Resolve
external edits explicitly to protect your work.

Next: [editor](editor.md), [project audio](audio.md#project-audio-clips), and [export
revisions](export.md#revision-and-timing).
