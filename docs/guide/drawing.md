# Procedural drawing

Use custom drawing when built-in objects cannot express the geometry you need. Create an
object with `k.object`, provide bounds, and draw through a local context. For reusable
properties, methods, simulation, or both 2D and 3D drawing, see [custom
objects](custom-objects.md).

## Draw in local coordinates

The object transform is applied by the engine. Draw around the local origin and keep the
object's bounds consistent with the visible geometry. The [drawing
project](../examples/drawing.md) creates a ribbon from a curve and restores property
values from a snapshot.

<<< ../../examples/lua/drawing/scenes/intro.lua

The context is available only during the draw callback. Do not store it for later or
mutate scene data while drawing. Drawing can run at arbitrary seek times and must not
advance animation or simulation.

## Paths and paint

Paths support moves, lines, quadratic and cubic curves, arcs, and closing a contour.
Paint can be a color, linear or radial gradient, image pattern, or text. Use `save` and
`restore` to scope transforms and clipping. A failed callback does not apply partial
drawing commands.

Bounds are local and also drive selection, anchors, and layout. Include the full area
used by strokes, curves, and effects. For a changing shape, compute bounds from the same
evaluated properties as the drawing.

## 3D custom geometry

For custom 3D drawing, create an immutable mesh with `k.mesh` and draw it using
`Draw3D`. Build the mesh outside the callback and reuse it. Supply instance matrices to
draw the same mesh more than once. See the [custom object
guide](custom-objects.md#_3d-drawing-and-immutable-resources) and
[Draw3D](../api/Draw3D.md).

For GPU effects and shaders, see [shaders](shaders.md).
