# Cameras

A canvas camera determines the view rendered in the video. The editor camera is for
inspecting the scene and does not affect the result. To animate a camera in your video,
animate the canvas properties in Lua.

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

<<< ../../examples/lua/camera/scenes/2d.lua

## 3D camera

`s:world_3d()` returns the built-in 3D canvas. The default camera is at `(0,0,3)`,
looking along local negative Z, with positive Y pointing up. Position uses `vec3`, and
rotation uses a quaternion.

| Property | Purpose |
| --- | --- |
| `camera_position` | Position in canvas space. |
| `camera_rotation` | View orientation. |
| `camera_mode` | `Perspective` or `Orthogonal`. |
| `camera_fov` | Vertical field of view in radians, or orthographic view height. |
| `camera_near` / `camera_far` | Clipping planes. |

See [Canvas3DOptions](../api/Canvas3DOptions.md) for defaults and limits. Keep `0 < near
< far`. In perspective mode, FOV must be greater than zero and less than pi. In
orthographic mode, view height must be positive.

## Aim with focus_camera

`canvas:focus_camera {position=vec3,target=vec3}` prepares an animation of the camera's
position and orientation toward a target. Use distinct, finite points. The receiver must
be a canvas with a 3D camera.

<<< ../../examples/lua/camera/scenes/intro.lua

The method returns a reusable definition. Call `play` to schedule it. It does not add a
permanent constraint, so the camera will not follow a target that moves later. For
continuous derived control, calculate orientation from properties or use
[signals](signals.md), following each callback's restrictions.

## Perspective and orthographic views

In perspective mode, objects appear smaller as they move farther away. In orthographic
mode, their apparent size depends on view height instead of distance. Switching modes is
immediate, and `camera_fov` changes meaning with the mode. Do not interpolate the same
numeric value across modes as if it represented the same view.

```lua
world.camera_mode:set("Orthogonal")
world.camera_fov:set(6)
```

This sets an orthographic view six units high. Width depends on the canvas aspect ratio.
`set` applies immediately during construction. To change modes at a scheduled time, use
an instant animation.

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
