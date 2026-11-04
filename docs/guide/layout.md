---
pageClass: guide-page
---

# Groups and layout

Use a group for a shared transform with manually placed children. Use a container when
the children should be arranged automatically from their sizes, `gap`, and `direction`.
Both keep children in the scene hierarchy.

## Arrange children

A horizontal container places children along X. Vertical layout uses Y. Reverse directions
reverse the arrangement without changing the objects' own dimensions.

<GuideDemo name="layout-directions" caption="Left: horizontal. Center: vertical. Right: horizontal reverse." still>

<!-- demo: layout-directions -->
<<< ./projects/layout/scenes/01-layout-directions.lua

</GuideDemo>

Red, green, and blue retain their insertion order. Direction controls their positions.
Try doubling `gap`. Increasing a child's `size` also moves the following children.
Nest containers to build rows inside columns. A group lets you position children freely.
[Container3D](../api/Container3D.md) also supports the depth axis.

## Fade a complete composition

Opacity on a container composites its complete subtree once. Individual child opacity
blends each child separately, which changes overlap regions.

<GuideDemo name="layout-opacity" caption="Left: one half-opacity group. Right: three half-opacity children." still>

<!-- demo: layout-opacity -->
<<< ./projects/layout/scenes/02-layout-opacity.lua

</GuideDemo>

On the left, the blue circle covers the earlier circles before the group fades. On the right,
the individual colors contribute in the overlap. Try animating the group opacity to zero when
fading a card with several overlapping decorations.
