---
pageClass: guide-page
---

# Objects and composition

## Create and attach objects

A constructor such as `s:circle {...}` creates data and returns a typed handle. The
object starts inactive. It joins scene evaluation, drawing, and selection only after you
attach it to an active tree:

Use `add` at the time the object should appear and `remove` when its lifetime should
end. The following scene demonstrates both operations.

## Watch a lifetime

Creating an object is separate from activating it. This scene is empty for one second,
shows the circle for one second, then becomes empty again.

<GuideDemo name="object-lifetime" caption="The circle is active only between seconds one and two.">

<!-- demo: object-lifetime -->
<<< ./projects/objects/scenes/01-object-lifetime.lua

</GuideDemo>

Seek backward to 1.5 seconds: the circle returns. Removal ends its lifetime rather than
deleting the entity. Try moving `add` before the first wait to make it visible at zero.

Use [coordinates and anchors](coordinates.md) to reason about local transforms, and
[groups and layout](layout.md) to arrange and fade compositions.

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

## Control an overlap

Sibling order determines drawing when 2D objects share the same stacking priority.
Change the order after attaching to choose which silhouette covers the other.

<GuideDemo name="object-order" caption="Blue covers white on the left. Moving white to the top reverses the overlap on the right." still>

<!-- demo: object-order -->
<<< ./projects/objects/scenes/02-object-order.lua

</GuideDemo>

The circles keep their local coordinates. `move_to_top` changes order among siblings.
It does not move geometry. Try reparenting the white circle to another transformed group to see why
preserving a local position can change its visible position.

## Coordinates and transforms

Objects use local coordinates relative to their parent. Moving or rotating a group
transforms its children together. Learn how axes, rotations, and inherited transforms
work in [coordinates and anchors](coordinates.md).

## Bounds and anchors

Bounds describe local geometry. Anchors help align it in parent coordinates. See the
[anchor comparison](coordinates.md#bounds-and-anchors-use-different-spaces) before
aligning objects with different sizes.

## Groups, containers, and layout

Use a group for manually placed children with a shared transform, or a container to
arrange them automatically. The [layout guide](layout.md) demonstrates directions,
gaps, and ordering with the same set of objects.

## Opacity and composition

A container's opacity composites its whole subtree once. Fading overlapping children
individually produces a different result. Compare both in
[groups and layout](layout.md#fade-a-complete-composition).

Use [canvases and projections](canvases.md) to combine 2D and 3D through rendered
textures. See the [Object reference](../api/Object.md) for hierarchy queries and methods.
