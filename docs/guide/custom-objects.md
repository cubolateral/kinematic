# Custom objects

Custom objects let you define a scene object in Lua when a built-in shape cannot
represent what you need to draw or simulate. A definition groups properties, methods,
and callbacks. Each instance still uses Kinematic’s hierarchy, animation, lifetime, and
selection.

Use a built-in shape when it fits the geometry, and a group to combine existing objects.
Use `k.object` when you need custom drawing, animatable parameters, or simulated state.
A regular Lua function can schedule a sequence, so it does not need a custom object.

## Define, instantiate, and attach

There are three steps:

1. `k.object {...}` creates a reusable definition without adding anything to the scene.
2. `s:object(definition, options)` creates an instance with its own properties and state.
3. `world:add(instance)` activates the instance in the hierarchy at the current scheduled time.

```lua
local marker = k.object {
    name = "Marker",
    dimension = 2,
    bounds = function()
        return {x = -20, y = -20, width = 40, height = 40}
    end,
    draw = function(ctx, properties)
        ctx:circle(0, 0, 20, properties.fill)
    end,
}

return function(s)
    local object = s:object(marker, {
        id = "marker",
        fill = k.color("#008CFF"),
    })
    s:world_2d():add(object)
    object:by {position = {x = 200}}:play(1)
end
```

`name` sets the custom type’s default name. `dimension` selects 2D or 3D and defaults to
2. Instance options can include an ID, display name, transforms, and resources supported
by that dimension. IDs are unique within a scene. Reusing a definition does not share
mutable state between instances.

## Properties with k.component

A component declares animatable fields. The `default` value determines each field’s
type. `min` and `max` constrain numbers, while `choices` restricts strings. Field names
cannot shadow built-in object fields.

```lua
local progress = k.component {
    value = {default = 0, min = 0, max = 1},
    style = {default = "Solid", choices = {"Solid", "Outline"}},
    offset = {default = k.vec2(0, 0)},
}

local meter = k.object {
    components = {progress},
    bounds = function()
        return {x = -200, y = -12, width = 400, height = 24}
    end,
    draw = function(ctx, properties)
        ctx:rect(-200, -12, 400 * properties.value, 24, properties.fill)
    end,
}
```

Pass components in `components={...}`. After instantiation, `object.value` is a property
handle like `object.position`. Use `get`, `set`, `to`, `by`, `from`, and `signal` as
supported by the type. Vectors keep their channels, for example
`object.offset.x:to(20):play(1)`.

Dynamic fields appear in Inspector and Timeline. Defaults, limits, and choices are
validated. In the draw callback, `properties.value` is the evaluated value, not a
handle. Do not call `properties.value:get()` there.

## Custom methods

`methods` takes functions whose first argument is the instance. A method can read
parameters, calculate values, and return an animation definition:

```lua
methods = {
    advance = function(self, amount)
        return self:by {value=amount}
    end,
}
```

After instantiation, call `object:advance(0.25):play(1)`. The method prepares the
definition, and `play` schedules it. Custom methods cannot replace fundamental
operations and are not copied onto animation definitions. For example,
`object:advance(0.25):advance(0.25)` is not valid.

LuaLS can infer properties and methods from Lua definitions, including imported ones.
Dynamically constructed names may need annotations. Explicit declarations make an object
easier to use and maintain.

## Bounds, selection, and anchors

The `bounds(properties,state)` callback returns local coordinates and dimensions. In 2D,
provide `x`, `y`, `width`, and `height`. In 3D, also provide `z` and `depth`. The box
origin does not need to be `(0,0)`.

Bounds must cover the geometry. They are used for selection, layout, and anchor queries.
If a property changes the drawn size, use it when calculating the bounds too. Do not
apply the object transform inside bounds. The engine composes the hierarchy.

In the [runnable project](../examples/custom-object.md), width and height control both
drawing and bounds. Progress changes the filled area without changing the selectable
area.

## 2D drawing

`draw(ctx,properties,state)` receives a [Draw2D](../api/Draw2D.md) context, a copy of
evaluated properties, and a copy of the state. Draw in local coordinates. The engine
applies position, scale, rotation, opacity, clipping, and composition.

<<< ../../examples/lua/custom-object/scenes/intro.lua

The example exposes `value`, `width`, and `height`, reports matching bounds, and uses
`advance` to prepare a relative animation. Both instances use the same definition with
independent values. `reset():play(1)` restores construction properties. It does not
reset simulation state.

The context supports paths, colors, gradients, images, and text. Pair `save` or `layer`
operations with `restore`. The context exists only during the callback, so do not
capture it for later use. Drawing commands are applied only if the callback succeeds.
See [procedural drawing](drawing.md).

## 3D drawing and immutable resources

With `dimension=3`, the callback receives [Draw3D](../api/Draw3D.md). Create geometry
with `k.mesh`, transform it with `k.mat4`, and draw it with `ctx:mesh` or
`ctx:instances`. Materials and uniforms belong to the instance.

<<< ../../examples/lua/custom-object/scenes/mesh.lua

The mesh is created once outside the callback. The drawing provides three matrices for
the same geometry. Object transforms, material, and rotation do not require rebuilding
the mesh. Bounds include all three instances. Triangle indices start at 1.

Drawing can capture immutable resources such as images, fonts, meshes, and matrices. Do
not copy buffers or reload files every frame. You can set a custom mesh shader in the
instance options. It replaces built-in lighting. See [shaders](shaders.md).

## State and update

Properties represent animatable values. `state` stores data accumulated by a simulation,
such as an integrated position, particles, or a random generator. Declare an initial
state and `update(state,frame,inputs)` to advance it in fixed steps.

```lua
local moving = k.object {
    state = {x=0},
    inputs = {speed="speed"},
    components = {k.component {speed={default=60, min=0}}},
    update = function(state, frame, inputs)
        state.x = state.x + inputs.speed * frame.dt
    end,
    bounds = function() return {x=-300,y=-20,width=600,height=40} end,
    draw = function(ctx, properties, state)
        ctx:circle(state.x - 200, 0, 12, properties.fill)
    end,
}
```

Each instance gets a copy of the initial state. `object:state()` returns another copy
for inspection. Editing that copy does not change the simulation. Use `mutate` to
schedule a change, `update()` for an explicit step, and `checkpoint()` to capture a
point that can speed up seeking. `auto_update` controls automatic steps.

`reset` and property snapshots do not replace state and checkpoints. See the [simulation
guide](simulation.md) for temporal reconstruction and history invalidation.

## Inputs and isolation

Declare external dependencies in `inputs`. These can be local property names, property
handles, other simulated objects, or audio players. Use `object:inputs({...})` to
configure them after instantiation. Dependencies are sampled at each step, and cycles
are rejected.

When omitted, inputs conservatively include the object’s own properties. `inputs={}`
declares no dependencies. This affects checkpoint invalidation: changing a visual
property that is not an input can preserve history, while changing a logic input
requires replay.

Update and mutate cannot capture mutable external values. Put changing data in `state`
and declare dependencies through inputs. Helper modules used by callbacks follow the
[simulation](simulation.md) restrictions. For reproducible randomness, keep
`k.random(seed)` in the state.

## Callback responsibilities

| Callback | Responsibility | Main restriction |
| --- | --- | --- |
| `bounds(properties,state)` | Return the local bounds. | Keep bounds consistent with the geometry. |
| `draw(ctx,properties,state)` | Produce drawing commands. | Do not change animation, schedule work, or advance state. |
| `update(state,frame,inputs)` | Advance state in fixed steps. | Use only state and declared dependencies. |
| `mutate(function(state) ... end)` | Apply a state operation at its scheduled time. | Follow the same isolation rules as update. |
| `signal(callback)` | Adjust the frame visually after evaluation. | Writes are temporary. Do not construct or schedule. |

Use [signals](signals.md) for a visual change that depends only on time. Use simulation
when a change depends on history. Drawing must never determine simulation results.
Seeking and exporting should evaluate the same scene.

## Reference and troubleshooting

See [ObjectOptions](../api/ObjectOptions.md) and
[PropertyDefinition](../api/PropertyDefinition.md) for options. Instance APIs are
[Script2D](../api/Script2D.md) and [Script3D](../api/Script3D.md).
[Object](../api/Object.md) lists shared methods and their requirements.

If an object is missing, check attachment and lifetime before debugging drawing. If
selection does not match the image, review bounds. If a seek produces a different
result, check state, inputs, and randomness. If a callback fails, read the diagnostic. A
failed candidate does not replace the last valid preview. Also validate with `--render`,
since `--check` does not exercise the GPU.
