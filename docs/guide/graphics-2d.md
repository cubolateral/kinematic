---
pageClass: guide-page
---

# 2D graphics

## Build a card and soften its edge

A rectangle can combine rounded corners, a fill, and an independent outline. Animate
one filter at a time to see how it changes the silhouette.

<GuideDemo name="shape-filter" caption="The rounded card blurs and becomes sharp again.">

<!-- demo: shape-filter -->
<<< ./projects/graphics-2d/scenes/01-shape-filter.lua

</GuideDemo>

Blur softens the outline as well as the fill and extends beyond the original edge.
Try `shadow_offset = k.vec2(12, 12)`, `shadow_blur = 8`, and a nontransparent
`shadow_color` in the constructor for a shadow instead. Compare at final output size.
See the dedicated [images guide](images.md) for photos and pixel sampling.

## Shapes and lines

Use `circle`, `ellipse`, `rect`, `triangle`, `regular_polygon_2d`, and `line_2d` to create
built-in geometry. `square` is a regular polygon preset. Each constructor has its own
options, listed under [k](../api/k.md). Check the type reference before using
fields from another shape.

Filled shapes use `fill`, while outlines use `stroke` and `stroke_width`. Transforms use
`position`, `rotation`, `scale`, and `opacity`. `rect.round` is a quad that sets the
corner radii. See [Panel](../api/Panel.md) and [PanelOptions](../api/PanelOptions.md) for
defaults and limits.

```lua
local card = k.panel{
    size = k.vec2(320, 160),
    round = k.quad(12, 12, 12, 12),
    fill = k.color("#0A0A0A"),
    stroke = k.color("#FFFFFF"),
    stroke_width = 2,
}
k.world_2d():add(card)
```

`triangle` creates a dedicated [Triangle](../api/Triangle.md). Its `size` sets the base
width and height. The animatable `apex` ranges from `0` (tip above the left end of the
base) to `1` (above the right end), defaulting to `0.5` (centered). The triangle points
up by default; use `rotation` to change its direction. A centered apex creates an
isosceles triangle; set height to width times `math.sqrt(3) / 2` for an equilateral one.

```lua
local triangle = k.triangle{ size = k.vec2(240, 180), apex = 0.5 }
k.world_2d():add(triangle)
triangle:to { apex = 1 }:play(1)
```

Previously, `triangle` was a regular polygon preset pointing right. It now points up
and no longer exposes `sides`. Use `regular_polygon_2d { sides = 3 }` for the previous
geometry or to animate the number of sides. `triangle_3d` remains a polygon preset.

The [layout example](../examples/layout.md) animates rectangles while keeping their
bottom edges aligned.

## Images

Load an immutable resource with `k.load_image`, then display it with `k.image_2d` or
`k.image_3d`. The dedicated [images guide](images.md) covers dimensions, proportions,
style, transparency, animation, and source pixel sampling with supplied photos.
Use [canvases and projections](canvases.md) for a live rendered texture.

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
