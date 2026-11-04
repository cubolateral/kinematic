---
pageClass: guide-page
---

# Cameras

A canvas camera determines the view rendered in the video. The editor camera is for
inspecting the scene and does not affect the result. To animate a camera in your video,
animate the canvas properties in Lua.

## Inspect a detail, then restore context

A triangle, square, and circle start fully visible. The camera zooms into the triangle,
pans to the square and circle, then restores the full view. The shapes stay fixed.

<GuideDemo name="camera-pan" caption="The camera visits the red triangle, green square, and blue circle, then restores the full view.">

<!-- focus: camera-pan -->
```lua
s:world_2d():to { camera_position = triangle.position:get(), camera_zoom = 2.5, }:play(1)
s:world_2d():to { camera_position = square.position:get(), }:play(1)
s:world_2d():to { camera_position = circle.position:get(), }:play(1)
s:world_2d():to { camera_position = k.vec2(0, 0), camera_zoom = 1, }:play(1)
```

<template #source>

<!-- demo: camera-pan -->
<<< ./projects/cameras/scenes/01-camera-pan.lua

</template>

</GuideDemo>

`camera_position` places the selected detail at the center of the frame. Zoom magnifies
all content together. Animate both properties back to recover the original framing.
Editor navigation only changes inspection, while these canvas properties determine
the exported image.

## 2D camera

`s:world_2d()` returns the built-in 2D canvas. `camera_position` is the point at the
center of the view, `camera_zoom` controls magnification, and `camera_rotation` is the
angle in radians. Zoom must be positive, and all values must be finite.

```lua
local world = s:world_2d()
world:to {
    camera_position = k.vec2(150, 0),
    camera_zoom = 1.5,
    camera_rotation = 0.2,
}:play(2)
```

Moving the camera changes the view of every object on the canvas. It does not change
their local positions. A zoom of 1.5 magnifies the image without changing project
resolution. To restore the initial view, animate all three values back explicitly.

## Follow a visible trajectory

Use an animated track when the camera should follow a calculated curve. Here the same
`point(angle)` function places the path markers and computes the camera position.
Blue dots mark the ellipse at equal angular intervals.

<GuideDemo name="camera-path" caption="The camera follows the dotted ellipse, with blue dots marking its trajectory.">

<!-- focus: camera-path -->
```lua
s:world_2d():signal(function(canvas)
    canvas.camera_position:set(point(phase:get()))
end)

phase:to(math.pi * 2):play(2, "linear")
```

<template #source>

<!-- demo: camera-path -->
<<< ./projects/cameras/scenes/02-camera-path.lua

</template>

</GuideDemo>

The signal reads phase at the evaluated time, so seeking works without accumulating
camera movement. The path is visible only to explain the trajectory. You can remove
the markers without changing the camera. See [signals](signals.md) for derived motion.

## 3D camera

`s:world_3d()` returns the built-in 3D canvas. The default camera is at `(0,0,3)`,
looking along local negative Z, with positive Y pointing up. Position uses `vec3`, and
rotation uses a quaternion.

| Property | Purpose |
| --- | --- |
| `camera_position` | Position in canvas space. |
| `camera_rotation` | View orientation. |
| `camera_mode` | `perspective` or `orthogonal`. |
| `camera_fov` | Vertical field of view in radians, or orthographic view height. |
| `camera_near` / `camera_far` | Clipping planes. |

See [Canvas3DOptions](../api/Canvas3DOptions.md) for defaults and limits. Keep `0 < near
< far`. In perspective mode, FOV must be greater than zero and less than pi. In
orthographic mode, view height must be positive.

## Perspective and orthographic views

In perspective mode, objects appear smaller as they move farther away. In orthographic
mode, their apparent size depends on view height instead of distance. Switching modes is
immediate, and `camera_fov` changes meaning with the mode. Do not interpolate the same
numeric value across modes as if it represented the same view.

```lua
world.camera_mode:set("orthogonal")
world.camera_fov:set(6)
```

This sets an orthographic view six units high. Width depends on the canvas aspect ratio.
`set` applies immediately during construction. To change modes at a scheduled time, use
an instant animation.

## Compare perspective with orthographic framing

Place equal cubes at different depths. Perspective makes the farther cube smaller.
Orthographic projection keeps their apparent scale equal.

<GuideDemo name="camera-projection" caption="The farther blue cube is smaller in perspective, then matches white in orthographic view.">

<!-- demo: camera-projection -->
<<< ./projects/cameras/scenes/03-camera-projection.lua

</GuideDemo>

The mode switches immediately. In orthographic mode `camera_fov = 5` means a view five
units high. Try moving the blue cube farther back: its orthographic size stays fixed
until it crosses a clipping plane. Near and far planes bound visibility. They do not
scale geometry.

## Aim with focus_camera

`canvas:focus_camera {position=vec3,target=vec3}` prepares an animation of the camera's
position and orientation toward a target. Use distinct, finite points. The receiver must
be a canvas with a 3D camera.

The method returns a reusable definition. Call `play` to schedule it. It does not add a
permanent constraint, so the camera will not follow a target that moves later. For
continuous derived control, calculate orientation from properties or use
[signals](signals.md), following each callback's restrictions.

## Cameras on separate canvases

Each canvas has its own camera properties. An additional canvas can render a view that a
projection displays in another world. `s:view("2d")` and `s:view("3d")` select the
built-in worlds. They do not move objects between worlds or reconfigure additional
canvases.

Texture resolution determines its pixels. A projection's `size` determines how much area
it occupies. See [canvases and projections](canvases.md) for compositions across
dimensions.

## Troubleshoot framing

If an object is missing, check that it is attached to an active tree and review its
dimension, position, clipping planes, and camera direction. In 3D, check the lighting or
use an unlit material to separate lighting issues from framing. In 2D, check the zoom
and camera position.

Render the [camera project](../examples/camera.md) to compare its animated views. It
includes separate 2D and 3D camera examples.
