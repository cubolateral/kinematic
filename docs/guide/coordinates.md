---
pageClass: guide-page
---

# Coordinates, transforms, and anchors

Use local coordinates to build reusable compositions. Position a child relative to its
parent, then move or rotate the parent to transform the whole arrangement.

## Origin, axes, and units

In the default 2D view the origin is the center, positive X points right, and positive Y
points down. At camera zoom 1, logical canvas units correspond to output pixels. In 3D,
positive Y points up and the default camera looks along negative Z. Camera framing
controls the conversion from spatial units to pixels.

```text
2D:                  3D default view:
      (0, 0) → +X           +Y
         ↓                  ↑
        +Y                  · → +X    View into -Z.
```

## A child follows its parent

2D rotations are radians: `math.pi / 2` is a quarter turn. Scale multiplies local
geometry before the ancestor transforms are composed.

<GuideDemo name="local-transform" caption="Four white circles rotate and move together with their parent.">

<!-- demo: local-transform -->
<<< ./projects/coordinates/scenes/01-local-transform.lua

</GuideDemo>

The children keep their local positions at `(0, -128)`, `(128, 0)`, `(0, 128)`, and
`(-128, 0)`. Their visible positions change as the parent rotates and moves. `global_position`, `global_rotation`, `global_scale`, and
`global_matrix` return the composed transform. In 2D, `global_opacity` composes opacity.
Try changing the parent destinations to move the whole arrangement along a different path.

## Bounds and anchors use different spaces

`bounds()` returns a local box (`x`, `y`, `width`, `height`. Also `z` and `depth` in
3D). `box_size()` returns dimensions. The box can be offset from the local origin.
Anchors such as `left()` and `middle()` return points in parent coordinates.
`left_to(point)` returns the local position required to align the left anchor to a
point in that parent.

<GuideDemo name="anchor-edge" caption="Four circles align their facing edges to the sides of a blue rectangle." still>

<!-- demo: anchor-edge -->
<<< ./projects/coordinates/scenes/02-anchor-edge.lua

</GuideDemo>

Each circle uses the anchor facing the rectangle: `right_to(square:left())`,
`left_to(square:right())`, `bottom_to(square:top())`, or `top_to(square:bottom())`.
Try changing a radius: its facing edge stays aligned. Custom objects must report bounds matching
their drawing so these queries and selection work correctly.

In 3D, rotation uses [quaternions](graphics-3d.md#quaternions), with additional front,
back, and combined anchors. Continue with [groups and layout](layout.md).
