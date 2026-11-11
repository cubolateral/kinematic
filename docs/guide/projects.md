---
pageClass: guide-page
---

# Projects and scenes

## See how scenes form a video

If `intro.lua` lasts two seconds and `outro.lua` lasts three, manifest order produces:

```text
Project time:  0                 2                          5
Scenes:       [ Intro: 2 s.    ][ Outro: 3 s.              ]
Local time:    0 → 2             0 → 3
```

Add both relative paths to `scenes` in that order. Each scene starts with its own local
time zero. Project audio clips use absolute project time. Reverse the list to reverse
the order of the exported scenes.

To inspect reload behavior, change a circle's radius from `30` to `60` without saving.
After the pending update completes, Preview shows the larger circle. Next enter an
invalid radius such as `"large"`: the error appears, while the larger circle remains
from the last applied revision. Correct the value, wait for a successful update, then
save. Previewing, applying, and saving are separate operations.

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
the scene files and their order in the video. Each file returns a `function()`. The
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
return {accent = k.color("#4080FF")}
```

```lua
local colors = require("modules.colors")

return function()
    k.world_2d():add(k.circle{fill = colors.accent})

    k.wait(1)
end
```

`require("folder.module")` looks for `folder/module.lua` and `folder/module/init.lua`.
Imports are cached within each scene revision, and circular imports are errors.
`k` is a read-only API bound to the importing scene, including when called from a module.
Each scene has its own Lua VM and module cache, so the same module can create objects and
schedule work independently in multiple scenes. Return a definition or factory function
from reusable modules; returning an instance shares that instance within the same scene.
Simulations use a [restricted module environment](simulation.md).

Scene files return exactly one function. Use `return function() ... end`, or declare a
local named function and return it. The engine calls it without arguments; scene operations
are available through `k` throughout the module graph.

## IDs and object identity

Give important objects an explicit `id`, for example `k.circle {id="ball"}`. IDs are
unique within a scene and preserve identity when declarations are inserted or reordered.
Without an ID, identity depends on declaration order. Changing a built-in type or
dynamic schema may recreate incompatible data.

IDs preserve selection and reconciliation, but they do not make Inspector changes
persistent. Objects missing from a new revision are removed without accumulating tracks
from previous revisions.

## Reloads and revisions

The editor prepares a candidate revision from files saved on disk. It
replays any required simulation and validates resources and drawing before applying the
candidate between frames. Errors leave the last valid preview in place. Modules and
resources determine which scenes need to be reloaded. Long replays can be cancelled
between time slices.

Exports use the latest successfully applied revision. If a saved file contains an error, the
source may differ from the exported video. Check the editor status before
exporting. Further changes wait until an export finishes or is cancelled.

Editable timeline events are stored separately from Lua in `kinematic/scenes/`. For
example, `scenes/intro.lua` corresponds to `kinematic/scenes/scenes/intro.ron`. Audio
clips are stored in the manifest. Caches can be regenerated. Keep event metadata when
sharing edits that include it.

## Files and conflicts

Use **Files** to create, open, and rename files or folders. Renaming does not update
`require` statements or resource paths. Files open in your configured external editor. Save there to reload the scene.
Kinematic watches saved files and keeps the previous valid preview when a reload fails.
Project metadata saves use atomic replacement and check the on-disk version.

Next: [editor](editor.md), [project audio](audio.md#project-audio-clips), and [export
revisions](export.md#revision-and-timing).
