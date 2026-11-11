---
pageClass: guide-page
---

# Get started

This guide assumes you know basic programming. If Lua is new to you, the
[Lua conventions](lua.md) page covers the syntax used here.

Kinematic is an animation editor for projects written in Lua 5.4. It runs your scene
code, prepares animations, and evaluates the result at any point on the timeline.
Preview, pause, and seek through your animation before exporting it as an MP4.

## Requirements and installation

Download the Windows or Linux x64 build from [GitHub
Releases](https://github.com/cubolateral/kinematic/releases). Lua and FFmpeg are
included, so you do not need to install Lua or configure your PATH. On Linux, make the
downloaded file executable:

```sh
chmod +x kinematic-X.Y.Z-linux-x64
./kinematic-X.Y.Z-linux-x64
```

Replace `X.Y.Z` with the version you downloaded. Builds target Windows 10 22H2 or later,
and Linux with glibc 2.35 or later. You also need a graphical session with OpenGL 3.3,
desktop libraries, and an audio driver. macOS is not an official target for this release
process.

To build from source, see [contributing](contributing.md).

## Create and explore a project

1. Open Kinematic and select **Create project**. Choose a name, folder, resolution, and frame rate. New projects default to 1280 × 720 at 60 FPS.
2. In **Editor settings → External editor**, choose VS Code, VSCodium, or a custom graphical editor. Set its executable and test the project and file commands. Install LuaLS support in that editor. Open `scenes/intro.lua` from **Files**.
3. Watch the **Preview**, select objects in **Scene Tree**, and inspect their properties in **Inspector**.
4. Use **Timeline** to choose a time. Press Space to play or pause when focus is outside a text field.
5. Save the Lua file in your external editor. Kinematic reloads after about 400 ms without further file changes. Unsaved external edits do not affect the preview.

Guide previews use a 960 × 540 canvas at 30 FPS. Choose these values in Project
settings to match their framing, or keep your own project settings and compare behavior.

Take the [editor tour](editor.md) and learn about [project
reloads](projects.md#reloads-and-revisions).

## Create and attach a circle

Replace `scenes/intro.lua` with this scene. The file returns a function with no arguments.
`k` belongs to this scene and supplies values, object constructors, and scheduling functions.
A constructor creates an inactive object; `add` makes it visible at the current timeline time.

<GuideDemo name="first-circle" caption="A white circle at the center of the frame." still>

<!-- demo: first-circle -->
<<< ./projects/start/scenes/01-first-circle.lua

</GuideDemo>

The origin is at the center. Positive X points right and positive Y points down.
This is a static image, so it needs no duration or wait. Try doubling `radius` to make the circle twice as wide without changing its center.

## Give it an appearance

Change just the constructor to control geometry and style. The outline has its own color
and width, independently of the fill.

<GuideDemo name="first-style" caption="A circle with a white outline and transparent fill." still>

<!-- focus: first-style -->
```lua
k.circle{
    fill = k.color("#00000000"),
    stroke = k.color("#FFFFFF"),
    stroke_width = 8,
}
```

<template #source>

<!-- demo: first-style -->
<<< ./projects/start/scenes/02-first-style.lua

</template>

</GuideDemo>

The radius measures from center to edge. Change only `stroke_width` to `12` and compare
the border while keeping the radius and fill unchanged.

## Move over time

`to` describes a destination. It does not run an animation. `play(1)` schedules that
change over one second and advances the scene's scheduling cursor.

<GuideDemo name="first-motion" caption="The circle travels from left to right with smooth easing, then holds.">

<!-- focus: first-motion -->
```lua
ball:to { position = k.vec2(256, 0) }:play(1)
```

<template #source>

<!-- demo: first-motion -->
<<< ./projects/start/scenes/03-first-motion.lua

</template>

</GuideDemo>

The circle covers 512 units in one second, then holds for one second. Change
`play(1)` to `play(2)` to double the movement duration over the same distance. Use [easing](easing.md) to change speed within the movement.

## Combine movement and color

Put several destinations in one animation table to change position, radius, and fill together.

<GuideDemo name="first-combined" caption="The circle moves right, grows, and turns blue together, then holds.">

<!-- focus: first-combined -->
```lua
ball:to {
    radius = 128,
    position = { x = 256 },
    fill = k.color("#008CFF"),
}:play(1)
```

<template #source>

<!-- demo: first-combined -->
<<< ./projects/start/scenes/04-first-combined.lua

</template>

</GuideDemo>

All three properties change over one second. The whole scene lasts two seconds,
including the final pause. Try changing `play(1)` to `play(1/2)` to shorten the transition.

## Export your first MP4

Choose **Export → Export**. Open `output/<project name>.mp4` and check the movement,
color change, and final pause. Export uses the applied revision, including unsaved code
that has successfully updated the preview. Temporary Inspector changes are discarded.
Choose **Cancel** in the Export menu to stop.

```sh
kinematic --render path/to/project
```

See [export](export.md) for scene exports, audio, and resolution. The complete
[2D Shapes example](../examples/2d.md) remains available as a project to explore.

Continue with [animation](animation.md), then [sequences](sequences.md) and
[objects](objects.md). Explore [Text](text.md) and [LaTeX](latex.md) when adding labels
and formulas to your scenes.
