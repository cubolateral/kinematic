# Animation

## Properties, channels, and values

Each type exposes animatable properties generated from its runtime fields.
`ball.position` is a handle, while `ball.position:get()` returns a value. Use
`ball:get("position")` to read by name, or `ball:prop("position.x")` to access a
channel, including when a property name conflicts with a method.

`set` changes a value immediately during scene construction. Inside a signal, the change
is temporary and applies only to the evaluated frame. Types, limits, and choices are
validated. See the [API reference](../api/index.md) for each type's options.

## Prepare and schedule animations

`to`, `by`, and `from` return reusable animation definitions. Preparing one does not
change the object or the timeline. Call `play` to schedule it:

```lua
local step = ball:by {position = {x = 100}}

step:play(1)
step:play(1)
```

Each animation moves the ball by 100 units. Its implicit starting value is resolved at
the scheduled time. Reusing `to` repeats the absolute destination, while reusing `by`
repeats the increment.

| Operation | Example | Meaning |
| --- | --- | --- |
| `to` | `ball:to {radius=50}` | Absolute destination. |
| `by` | `ball:by {radius=10}` | Relative change. |
| `from` | `ball:from {radius={20,50}}` | Explicit start and end values. |

Omitted channels are not animated. For vector pairs, use
`position={k.vec2(0,0),k.vec2(100,50)}` or `position={x={0,100}}`. Overlapping
declarations in one table are errors.

## Duration and easing

`play()` and `play(0)` apply changes instantly. `play(seconds,easing)` uses a duration
in seconds and an easing curve named by a `snake_case` string. The default is
`"cubic_in_out"`. `"linear"` keeps a constant speed, while `"cubic_out"` slows near the end. See
[Easing](../api/Aliases.md#property-kinematic-easing) for the available curves.

```lua
ball:to {radius = 50}:play(1, "cubic_out")
```

The curve families are `quad`, `cubic`, `quart`, `quint`, `sine`, `expo`, `circ`,
`back`, `elastic`, and `bounce`. Each supports `_in` (accelerates at the start),
`_out` (decelerates at the end), and `_in_out` (both). Use `"linear"` for constant speed.
Only the documented spellings are accepted.

An animation definition exposes only `play`. Do not chain `to` or object methods on it.
To combine movements, schedule them in scopes.

## Sequences, parallel work, and loops

| Scope | Effect on the parent timeline |
| --- | --- |
| `s:chain(function() ... end)` | Schedules operations one after another. |
| `s:all(function() ... end)` | Starts child operations together and advances by the longest duration. |
| `s:parallel(function() ... end)` | Schedules operations without advancing or extending the parent. |
| `s:loop(function() ... end)` | Repeats a finite, positive-duration cycle without advancing the parent. |

```lua
s:all(function()
    ball:to {position = k.vec2(200, 0)}:play(2)
    ball:to {fill = k.color("#34D399")}:play(0.5)
end)

s:wait(0.5)
```

This example lasts 2.5 seconds. Inside `parallel`, operations run in sequence unless
grouped with `all`. Reserve time in the parent so parallel work appears in the video.

A `loop` callback runs once to prepare a cycle that repeats for the object's lifetime.
Do not create, attach, or remove objects, or schedule audio, inside the cycle. Nested
loops and ambiguous concurrent writes in repeated scopes are rejected. Do not use a Lua
`while true` loop for animation.

## Timeline events and independent tracks

`s:event("Reading")` creates an editable pause on the timeline. `s:wait(seconds)`
creates a fixed pause. Use events to adjust pacing without changing values in code.

`s:track(initial,id)` creates an independent property. It supports `get`, `set`, `to`,
`by`, `from`, and `signal`, including vector channels. Use it for a shared phase or
parameter that drives signals. The optional ID preserves its identity across reloads.

## Snapshots and time

`object:snapshot()` captures animatable fields, dynamic properties, and uniforms.
`restore(snapshot):play(1)` animates the object back to that snapshot. `save()` pushes a
snapshot, and `restore():play(1)` pops it. `reset():play()` uses the constructor's
initial values. A snapshot belongs to the object that created it.

<<< ../../examples/lua/drawing/scenes/intro.lua

The scene is evaluated at the requested time without drawing earlier frames. Removals
happen over time, and simulations rebuild state with fixed steps and checkpoints.
Drawing never advances animation. Learn more about [simulation](simulation.md) and
[composition](objects.md).
