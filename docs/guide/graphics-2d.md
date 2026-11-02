# 2D graphics

## Shapes and lines

Use `circle`, `ellipse`, `rect`, `regular_polygon_2d`, and `line_2d` to create built-in
geometry. `triangle` and `square` are polygon presets. Each constructor has its own
options, listed under [Scene](../api/Scene.md). Check the type reference before using
fields from another shape.

Filled shapes use `fill`, while outlines use `stroke` and `stroke_width`. Transforms use
`position`, `rotation`, `scale`, and `opacity`. `rect.round` is a quad that sets the
corner radii. See [Rect](../api/Rect.md) and [RectOptions](../api/RectOptions.md) for
defaults and limits.

```lua
local card = s:rect {
    size = k.vec2(320, 160),
    round = k.quad(12, 12, 12, 12),
    fill = k.color("#1E293B"),
    stroke = k.color("#38BDF8"),
    stroke_width = 2,
}
s:world_2d():add(card)
```

The [layout example](../examples/layout.md) animates rectangles while keeping their
bottom edges aligned.

## Images

`k.image("assets/photo.png")` loads an immutable resource for the current revision. Add
it to a scene with `s:image_2d {source=image}`. Asset paths are relative to the project
and remain fixed during export. Images are limited to 64 MiB and 16 million decoded
pixels. `get_pixel_color` requires an image source and reads a local position.

Use `source=canvas:texture()` with projections to display content rendered on another
canvas. Canvas dependencies are checked. Cycles and references between scenes are
rejected.

## Text and code objects

Text, formulas, and code have dedicated guides because their transitions, layout, and
queries need additional explanation:

- [Text](text.md): fonts, alignment, morph, fade, and the differences between 2D and 3D.
- [LaTeX](latex.md): syntax, mathematical geometry, transitions, bounds, and compatibility.
- [Code](code.md): syntax highlighting, ranges, search, selection, and animated edits.

## Filters

Supported objects expose blur, shadow, and other filter fields in their generated
options. See [CircleOptions](../api/CircleOptions.md) for an example. Animate filters
through the same property channels as other values. Check their cost at the final output
size, since capture and blur can increase render time.

## Cameras and composition

The 2D world is a Canvas2D. Its camera determines the rendered view, while the editor
camera is for inspection only. See [cameras](cameras.md) for position, zoom, and
rotation, and [canvases and projections](canvases.md) for displaying content from
another view.

For custom Lua drawing, see [custom objects](custom-objects.md) and [procedural
drawing](drawing.md). For GPU programs, see [shaders](shaders.md).
