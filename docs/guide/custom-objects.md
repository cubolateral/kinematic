---
pageClass: guide-page
---

# Custom objects

Use `k.object` when a built-in shape or group cannot express the drawing or behavior
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
local progress = k.component {
value = { default = 0, min = 0, max = 1 },
}

local meter = k.object {
name = "Meter",
components = { progress },

bounds = function()
    return { x = -256, y = -8, width = 512, height = 16 }
end,

draw = function(ctx, properties)
    ctx:rect(-256, -8, 512, 16, k.color("#0A0A0A"))
    ctx:rect(-256, -8, 512 * properties.value, 16, properties.fill)
end,
}
```

<template #source>

<!-- demo: custom-meter -->
<<< ./projects/custom-objects/scenes/01-custom-meter.lua

</template>

</GuideDemo>

`k.object` creates a definition, not a visible object. `s:object(meter, options)` creates
an instance. `world:add(instance)` attaches it at the current timeline time. Expand
**Complete scene** above to see both instances and their animations.

## Animate the exposed property

The instance exposes `value` as a property handle, just like `position`. The example
starts both animations together, and `all` waits for the longer one:

```lua
s:all(function()
    first.value:to(1):play(2, "linear")
    second.value:to(0.5):play(1, "linear")
end)
```

Inside `draw`, `properties.value` is a number, not a handle. Outside the callback,
`first.value` supports `get`, `set`, `to`, `by`, and `from`. Dynamic properties also
appear in Inspector and Timeline.

The `default` determines the property type. Numeric limits use `min` and `max`.
String choices use exact `snake_case` values. For example:

```lua
local appearance = k.component {
    style = {default = "solid", choices = {"solid", "outline"}},
    offset = {default = k.vec2(0, 0)},
}
```

Include additional components in the definition's `components` table and use their
values in `draw`. Names must not shadow built-in fields. Vector properties keep their
channels, such as `object.offset.x`.

## Add a named operation

If a sequence repeatedly advances the meter, add this `methods` field to the same
`k.object` definition:

```lua
methods = {
    advance = function(self, amount)
        return self:by {value = amount}
    end,
},
```

Then `first:advance(0.25):play(1)` schedules an increment. The method returns an
animation definition, so `play` remains explicit. A plain Lua helper is sufficient
when an operation does not belong to a reusable type.

## Keep drawing local

`draw(ctx, properties, state)` reads evaluated values and emits drawing commands.
The engine applies position, rotation, scale, and opacity. Do not apply these
transforms a second time inside the callback, schedule animations there, or update
simulation state while drawing.

`bounds(properties, state)` must cover the geometry in the same local coordinates.
If a parameter changes the object's dimensions, use it in both drawing and bounds.
An incorrect box affects selection, anchors, and layout.

The drawing context exists only during the callback. Pair `save` or `layer` with
`restore`, and load immutable resources outside it. See [procedural drawing](drawing.md)
for paths, gradients, and clipping.

## Reuse and extend

A module can return the meter definition so several scenes can instantiate it. Each
instance keeps independent properties and state. Supply an `id` when you need stable
identity across reloads. IDs must be unique within the scene.

For 3D, set `dimension = 3`, return bounds with `z` and `depth`, and use the
[Draw3D](../api/Draw3D.md) context with immutable meshes and matrices. Keep mesh
construction outside `draw`.

For accumulated motion, declare `state`, `inputs`, and an `update` callback as shown in
[simulation](simulation.md). A custom drawing alone does not need simulated state.
See [ObjectOptions](../api/ObjectOptions.md) for definition options and
[Script2D](../api/Script2D.md) for instance methods.
