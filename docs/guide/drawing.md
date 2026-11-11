---
pageClass: guide-page
---

# Procedural drawing

Use custom drawing when built-in objects cannot express the geometry you need. Create an
object with `k.object_2d` or `k.object_3d`, provide `on_bounds`, and draw through a local context. For reusable
properties, methods, simulation, or both 2D and 3D drawing, see [custom
objects](custom-objects.md).

## Draw in local coordinates

The object transform is applied by the engine. Draw around the local origin and keep the
object's bounds consistent with the visible geometry.

The context is available only during the `on_draw` callback. Do not store it for later or
mutate scene data while drawing. Drawing can run at arbitrary seek times and must not
advance animation or simulation.

## Close and fill a path

Construct a contour in local coordinates, close it, and choose fill and stroke
independently. The bounds include the line thickness and the curve's control region.

<GuideDemo name="drawing-path" caption="A closed quadratic path has a left-to-right gradient and a white outline.">

<!-- demo: drawing-path -->
<<< ./projects/drawing/scenes/01-drawing-path.lua

</GuideDemo>

The curve starts at `(200, 100)`, bends toward control point `(0, -150)`, and ends at
`(-200, 100)`. The control point guides the tangent. It need not lie on the curve.
Try changing only the control Y coordinate to `-250`, then update bounds if needed.
`save`/`restore` keep local transforms and clipping scoped to their intended drawing.
See [Draw2D](../api/Draw2D.md) for the exact gradient and clipping signatures.

## Paths and paint

Paths support moves, lines, quadratic and cubic curves, arcs, and closing a contour.
Paint can be a color, linear or radial gradient, image pattern, or text. Use `save` and
`restore` to scope transforms and clipping. A failed callback does not apply partial
drawing commands.

Bounds are local and also drive selection, anchors, and layout. Include the full area
used by strokes, curves, and effects. For a changing shape, compute bounds from the same
evaluated properties as the drawing.

## Scope clipping and transforms

Save the context before clipping so later drawing can use the full area again.

<GuideDemo name="drawing-clip" caption="A triangular clip trims the red rectangle and the translated blue circle." still>

<!-- demo: drawing-clip -->
<<< ./projects/drawing/scenes/02-drawing-clip.lua

</GuideDemo>

The circle's translation is scoped to the saved context. Restoring removes both clipping
and translation, so the white outline uses the original coordinates. Try drawing one
extra circle after restore to verify that it is no longer clipped.

## 3D custom geometry

For custom 3D drawing, create an immutable mesh with `k.mesh` and draw it using
`Draw3D`. Build the mesh outside the callback and reuse it. Supply instance matrices to
draw the same mesh more than once. See the [custom object
guide](custom-objects.md#reuse-and-extend) and
[Draw3D](../api/Draw3D.md).

For GPU effects and shaders, see [shaders](shaders.md).
