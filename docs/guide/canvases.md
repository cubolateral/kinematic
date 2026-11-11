---
pageClass: guide-page
---

# Canvases and projections

A canvas is an independent view with its own camera and texture. A projection displays
that texture as an object in another world. Use this to place 3D content in a 2D
composition or show a 2D panel in 3D space.

## Display a 2D panel in 3D

Render a circle into a 2D canvas, then use that texture on a spatial rectangle.
The dark panel distinguishes the source canvas from the surrounding 3D view.

<GuideDemo name="canvas-panel" caption="A blue circle rendered in 2D appears on a rotating panel in 3D.">

<!-- demo: canvas-panel -->
<<< ./projects/canvases/scenes/01-canvas-panel.lua

</GuideDemo>

The source canvas has a 16:9 ratio. Increase only projection size to enlarge the
panel in space, or source resolution to add texture pixels. Changing the camera of the
source changes the image on the panel. Changing the destination camera changes how the
whole panel is viewed.

## Trace the source to the screen

Build the source first, then connect its texture to an object in the destination world.

```text
Root → Canvas3D → Group (Sphere + Text3D) → Texture → Projection2D → 2D world → Frame.
```

<GuideDemo name="canvas-projection" caption="A blue sphere and 3D label rotate together inside a 2D projection.">

<!-- demo: canvas-projection -->
<<< ./projects/canvases/scenes/02-canvas-projection.lua

</GuideDemo>

The sphere and label belong to a group in the source canvas. The projection belongs to the destination and
controls the displayed rectangle. Try reducing only the source resolution to `{80, 60}`:
the displayed dimensions stay the same, while texture detail decreases. Match both aspect
ratios to preserve proportions. Reverse the dimensions using Canvas2D and Projection3D
for a panel in space.

## The root, worlds, and additional canvases

Every scene has an internal root and the `world_2d()` and `world_3d()` canvases.
Ordinary objects belong to a world or group of the same dimension. Attach additional
canvases to the root:

```lua
local source = k.root():add(k.canvas_3d{resolution = {900, 450}})
source:add(k.cube{
    albedo = k.color("#008CFF"),
    unlit = true,
})

k.world_2d():add(k.projection_2d{
    source = source:texture(),
    size = k.vec2(900, 450),
})
```

Creating a canvas does not activate it. Attaching a canvas does not automatically
display its texture in every world. The projection is the object that displays it.

## Resolution and size

`resolution={width,height}` sets a canvas's fixed render target in pixels, independently
of project resolution. The camera determines which area of the world those pixels
represent. A projection's `size` determines its local dimensions in the destination
world.

Enlarging a projection without increasing its texture resolution can make it look less
sharp. Larger textures use more memory and rendering time, even when displayed in a
small area. Keep dimensions at or below 8192. Match the texture and projection aspect
ratios to avoid distortion.

## Live textures and revisions

`canvas:texture()` returns a handle to the canvas output. The image updates with the
evaluated time and camera properties. Resource relationships stay fixed for a revision.
Do not reload resources or copy pixels in Lua every frame.

Dependencies must belong to the same scene and form a graph without cycles. A canvas
cannot depend on a texture that eventually depends on that canvas. The engine evaluates
dependencies in order.

## Combine dimensions

[Projection2D](../api/Projection2D.md) displays a texture as a rectangle in the 2D
world. [Projection3D](../api/Projection3D.md) displays it in 3D space. The projection's
dimension belongs to the destination, so the source canvas can use the other dimension.

The [composition project](../examples/composition.md) displays a 3D texture in a 2D
world. The sphere and cube rotate and move in their source canvas, while the projection
keeps its framing in the destination.

## Combine with effects

Canvases can provide textures to projections and to samplers declared in a
[shader's](shaders.md) `textures` table. The shader receives the rendered output, not
the original hierarchy. Apply opacity at the level where you want the group composited.
A group's opacity composites the subtree once and preserves its internal overlaps.

If a texture is empty, check its dependencies, resolution, camera, and lifetime.
`--check` does not replace graphical validation of compositions and shaders.
