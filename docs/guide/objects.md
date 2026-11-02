# Objects and composition

## Create and attach objects

A constructor such as `s:circle {...}` creates data and returns a typed handle. The
object starts inactive. It joins scene evaluation, drawing, and selection only after you
attach it to an active tree:

```lua
local circle = s:circle {id = "circle", radius = 30}

s:wait(1)
s:world_2d():add(circle)

s:wait(1)
circle:remove()

s:wait(1)
```

Here, the circle appears from 1 to 2 seconds. `remove` ends the subtree's lifetime
without deleting its entities or hierarchy links. Seeking to an earlier time restores
it. `remove_children` ends the lifetimes of its children. Inactive objects are ignored
by evaluation, rendering, selection, and editor views.

## Hierarchy and draw order

Every scene has an internal root container, available through `s:root()`. The 2D and 3D
worlds are built-in canvases. Attach ordinary objects to the matching world and
additional canvases to the root. Objects must belong to the same scene, and the target
of `add` must accept children.

`children()` returns the ordered children, and `get_child(1)` returns the first. Use
`parent()`, `ancestor(type_name)`, and `type_name()` to inspect the tree.
`insert(child,index)` attaches a child at a specific position. Indices start at 1.

`move_to`, `move_up`, `move_down`, `move_to_top`, and `move_to_bottom` change the order
among siblings. In 2D, later children are drawn over earlier ones. In 3D, depth testing
also applies, so list order does not replace spatial depth.

`reparent(new_parent)` changes the parent across the entire timeline while preserving
local properties and lifetime. It does not animate a parent change. Since transforms are
local, changing an ancestor can change an object's visible position.

## Coordinates and transforms

The default 2D view is centered at the origin, with positive X to the right and positive
Y downward. Positions and sizes use logical canvas units. In 3D, positive Y points up
and the default camera looks along negative Z.

A local transform combines position, rotation, and scale with the transforms of its
ancestors. 2D rotation uses radians, and 3D rotation uses quaternions.
`global_position`, `global_rotation`, `global_scale`, and `global_matrix` return the
composed result. `global_opacity` is available in 2D.

Constructors accept complete values or partial channels. For example, `position={x=100}`
keeps the other default values. Vectors have `x/y/z` channels, colors have `r/g/b/a`,
and quads have `a/b/c/d`.

## Bounds and anchors

`bounds()` returns the local bounding box with `x`, `y`, `width`, and `height`. In 3D it
also includes `z` and `depth`. The box can be offset from the origin. `box_size()`
returns only its dimensions.

`middle`, `left`, `right`, `top`, `bottom`, and the corner anchors return points in
parent coordinates. 3D objects also provide `front`, `back`, and combined anchors. Each
anchor has a `*_to(point)` query that calculates the local position needed to place it
at the target:

```lua
circle.position:set(circle:left_to(k.vec2(-200, 0)))
```

Local bounds are not the same as anchors transformed into parent coordinates. Custom Lua
objects must report bounds that match their drawing so selection and layout work
correctly.

## Groups, containers, and layout

A [Group2D](../api/Group2D.md) or [Group3D](../api/Group3D.md) gives its children a
shared transform. A [Container2D](../api/Container2D.md) or
[Container3D](../api/Container3D.md) also lays out children using `gap` and `direction`.

```lua
local row = s:container_2d {gap = k.vec2(24, 0), direction = "Horizontal"}
s:world_2d():add(row)

row:add(s:circle {radius = 30})
row:add(s:rect {size = k.vec2(70, 60)})
```

Available directions, including reverse directions and the 3D depth axis, are listed in
the generated options. Layout uses the children’s sizes. Use a group when you want to
control positions yourself.

## Opacity and composition

A container's opacity composites its subtree once. This differs from applying the same
opacity to each overlapping child individually. To render a texture for reuse, combine a
canvas with a projection as shown in the [composition
project](../examples/composition.md).

The [2D shapes project](../examples/2d.md) demonstrates object lifetime and motion. The
[Object API reference](../api/Object.md) describes shared operations and their
requirements.
