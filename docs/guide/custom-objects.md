---
pageClass: guide-page
---

# Custom objects

Use `k.object_2d` or `k.object_3d` when a built-in shape or group cannot express the drawing or behavior
you need. A definition describes properties and callbacks. Each instance has its own
values and participates in the normal hierarchy, selection, and animation system.

This page builds a meter with one animated value. For objects that accumulate history,
continue with [simulation](simulation.md) after learning the drawing model here.

## Define a meter

A component declares `value` as a number between 0 and 1. The draw callback receives
its evaluated value and paints that fraction of a 512-unit track. Bounds describe the
whole meter, including its empty area, so selection does not shrink with the fill.

<GuideDemo name="custom-meter" caption="Two instances share a definition, but their progress is independent.">

<!-- focus: custom-meter -->
```lua
local progress = k.component("Progress", {
    value = {default = 0, min = 0, max = 1},
})

local meter = k.object_2d("Meter", {
    components = {progress},
    on_bounds = function(self)
        return k.rect(-256, -8, 512, 16)
    end,
    on_draw = function(self, ctx)
        ctx:rect(-256, -8, 512, 16, k.color("#0A0A0A"))
        ctx:rect(-256, -8, 512 * self.value, 16, self.fill)
    end,
})
```

<template #source>

<!-- demo: custom-meter -->
<<< ./projects/custom-objects/scenes/01-custom-meter.lua

</template>

</GuideDemo>

`k.object_2d` creates a callable definition. `meter(options)` creates an independent
instance, using the same construction pattern as `k.panel(options)`.
`world:add(instance)` attaches it at the current timeline time. Expand
**Complete scene** above to see both instances and their animations.

## Animate the exposed property

The instance exposes `value` as a property handle, just like `position`. The example
starts both animations together, and `all` waits for the longer one:

```lua
k.all(function()
    first.value:to(1):play(2, "linear")
    second.value:to(0.5):play(1, "linear")
end)
```

Inside `on_draw` and `on_bounds`, `self.value` is an evaluated number. Outside callbacks,
`first.value` supports `get`, `set`, `to`, `by`, and `from`. Dynamic properties also
appear in Inspector and Timeline. Every component takes its display name as the first
argument: `k.component("Progress", {...})`.
Object definitions follow the same pattern: `k.object_2d("Meter", {...})`.
The Inspector groups its fields under that name; object fields remain directly accessible.

The `default` determines the property type. Numeric limits use `min` and `max`.
String choices use exact `snake_case` values. For example:

```lua
local appearance = k.component("Appearance", {
    style = {default = "solid", choices = {"solid", "outline"}},
    offset = {default = k.vec2(0, 0)},
})
```

Include additional components in the definition's `components` table and use their
values in `on_draw`. Names must not shadow built-in fields. Vector properties keep their
channels, such as `object.offset.x`.

## Add a named operation

Define operations directly on the object definition, inside its declaration or afterward:

```lua
function meter:advance(amount)
    return self:by {value = amount}
end

function meter.width_for(value)
    return value * 512
end
```

`first:advance(0.25):play(1)` schedules an increment. The method receives the instance
handle and returns an animation definition. Helpers such as `meter.width_for(0.5)`
use ordinary Lua dot syntax and receive no implicit `self`. There is no `methods` table.
Functions must not replace fundamental operations or properties. The `on_` prefix is
reserved for the supported engine callbacks.

## Keep drawing local

`on_draw(self, ctx)` reads evaluated properties and `self.state`, then emits drawing commands.
The receiver and state tables are read-only in drawing and bounds callbacks.
The engine applies position, rotation, scale, and opacity. Do not apply these
transforms a second time inside the callback, schedule animations there, or update
simulation state while drawing.

`on_bounds(self)` returns a `k.rect(x, y, width, height)` covering the geometry in local coordinates.
Call `instance:on_bounds()` to query it directly; native objects expose the same operation.
Callbacks can also be defined afterward with `function meter:on_draw(ctx)` and
`function meter:on_bounds()`.
If a parameter changes the object's dimensions, use it in both drawing and bounds.
An incorrect box affects selection, anchors, and layout.

The drawing context exists only during the callback. Pair `save` or `layer` with
`restore`, and load immutable resources outside it. See [procedural drawing](drawing.md)
for paths, gradients, and clipping.

## Reuse and extend

A module can return the meter definition so several scenes can instantiate it. Each
instance keeps independent properties and state. Supply an `id` when you need stable
identity across reloads. IDs must be unique within the scene.

For 3D, use `k.object_3d`, return `k.box(x, y, z, width, height, depth)` from `on_bounds`, and use the
[Draw3D](../api/Draw3D.md) context with immutable meshes and matrices. Keep mesh
construction outside `on_draw`.

For accumulated motion, declare `state`, `inputs`, and an `on_update` callback as shown in
[simulation](simulation.md). A custom drawing alone does not need simulated state.
See [ObjectOptions](../api/ObjectOptions.md) for definition options and
[Script2D](../api/Script2D.md) for instance methods.
