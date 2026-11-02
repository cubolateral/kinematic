# Get started

Kinematic is an animation editor for projects written in Lua 5.4. It runs your scene
code, prepares animations, and evaluates the result at any point on the timeline.
Preview, pause, and seek through your animation before exporting it as an MP4.

## Requirements and installation

Download the Windows or Linux x64 build from [GitHub
Releases](https://github.com/cubolateral/kinematic/releases). Lua, LuaLS, and FFmpeg are
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
2. Open `scenes/intro.lua` in **Files**. **Code** displays the file and LuaLS help.
3. Watch the **Preview**, select objects in **Scene Tree**, and inspect their properties in **Inspector**.
4. Use **Timeline** to choose a time. Press Space to play or pause when focus is outside a text field.
5. Press `Ctrl+S` to save. Code changes update the preview after about 400 ms without typing. You do not need to save before previewing them.

Take the [editor tour](editor.md) and learn about [project
reloads](projects.md#reloads-and-revisions).

## Lua essentials

A scene returns one function. The `s` parameter represents the scene, and `k` provides
value and resource constructors. `local` declares a variable, `{}` creates a table, and
`:` calls a method with its object as the first argument.

```lua
return function(s)
    local ball = s:circle {
        radius = 30,
        fill = k.color("#38BDF8"),
    }
    s:world_2d():add(ball)

    ball:to {position = k.vec2(200, 0)}:play(2)
    s:wait(0.5)
end
```

`k.vec2` creates a value, while `s:circle` creates an object. `add` attaches the object
to the scene hierarchy and gives it a lifetime. `to` prepares an animation, and
`play(2)` schedules it for two seconds. `wait` reserves time without adding motion. Lua
does not run the application loop.

## Your first 2D animation

Follow the [2D Shapes example](../examples/2d.md). It creates a circle, adds it to the
2D world, and animates its position, color, and opacity.

<<< ../../examples/lua/2d/scenes/intro.lua

Try changing the destination to `k.vec2(0, -100)` and the duration to `1`. Compare
`"linear"` with `"cubic_in_out"`. See
[Easing](../api/Aliases.md#property-kinematic-easing) for the full list.

## Export your first MP4

When the preview is ready, choose **Export → Export**. The video is saved to
`output/<project name>.mp4`. Export uses the applied revision, even when the code has
not been saved. Temporary Inspector changes are discarded. Choose **Cancel** in the
Export menu to stop.

To export from the command line while a graphical session is available:

```sh
kinematic --render path/to/project
```

See [export](export.md) for details about audio, revisions, and resolution.

## A first look at 3D

Use `s:world_3d():add(...)` to add spatial objects and `s:view("3d")` to select the 3D
world. The camera looks along negative Z, with positive Y pointing up. Rotations use
quaternions:

```lua
return function(s)
    s:view("3d")
    local cube = s:cube {
        albedo = k.color("#38BDF8"),
        unlit = true,
    }
    s:world_3d():add(cube)

    cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3)
    s:wait(0.5)
end
```

Explore the [3D primitives project](../examples/3d.md), which displays a built-in cube
on its own canvas. Continue with [objects and composition](objects.md).
