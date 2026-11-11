---
pageClass: guide-page
---

# Animation

An animation describes how a property changes over time. Create an object, attach it,
and schedule a destination with `play`. Lua prepares the timeline. It does not run a
frame loop while the video plays.

## Properties, channels, and values

`ball.position` is a property handle. `ball.position:get()` reads its value. A channel
such as `ball.position.x` changes only one part of a vector. Use `set` for an immediate
construction value and `to` with `play` for a timed change:

```lua
ball.position.x:set(-200)

ball.position.x:to(200):play(2, "linear")
```

This moves from left to right over two seconds, leaving Y unchanged. `set` is not a
timed transition. For computed values at each evaluated frame, see [signals](signals.md).

## Prepare and schedule animations

`to`, `by`, and `from` prepare animation definitions. Nothing is scheduled until `play`.
A definition can be reused:

```lua
local step = ball:by {position = {x = 100}}

step:play(1)
step:play(1)
```

Each play adds 100 units. Its implicit starting value is resolved at the scheduled
time, rather than being frozen when `step` is created.

| Operation | Example | Meaning |
| --- | --- | --- |
| `to` | `ball:to {radius = 50}` | Move to an absolute destination. |
| `by` | `ball:by {radius = 10}` | Add a relative amount. |
| `from` | `ball:from {radius = {20, 50}}` | Supply both endpoints explicitly. |

Omitted channels keep their existing values. For example,
`ball:from {position = {x = {-120, 240}}}:play(2)` specifies X's endpoints while
preserving Y. Do not declare the same channel twice in one animation table.

## Compare destinations and increments

Both circles begin at X = -256. The white circle plays the same destination twice.
The blue circle plays the same increment twice. Watch what happens in the second
second.

<GuideDemo name="animation-modes" caption="Top: repeating to stays at zero. Bottom: repeating by adds another 256 units.">

<!-- focus: animation-modes -->
```lua
k.all(function()
    k.chain(function()
        target.position.x:to(0):play(1, "linear")
        target.position.x:to(0):play(1, "linear")
    end)
    k.chain(function()
        increment.position.x:by(256):play(1, "linear")
        increment.position.x:by(256):play(1, "linear")
    end)
end)
```

<template #source>

<!-- demo: animation-modes -->
<<< ./projects/animation/scenes/01-animation-modes.lua

</template>

</GuideDemo>

`chain` keeps each pair sequential. `all` starts the two chains together. The
[sequences guide](sequences.md) explains these scheduling scopes in detail.

## Duration and easing

`play(seconds, easing)` schedules a duration in seconds. `play()` and `play(0)` apply
the change instantly. The default easing is `"cubic_in_out"`. Use `"linear"` for a
constant speed or `"cubic_out"` to slow near the destination.

```lua
ball:to {radius = 50}:play(1, "cubic_out")
```

See [easing](easing.md) to compare curves. Easing names are exact `snake_case` strings.
An animation definition exposes `play`, not additional object methods: schedule the
next animation on the object itself.

## Sequences, parallel work, and loops

A normal `play` advances the scheduling cursor. `all` waits for its longest child.
`parallel` and `loop` leave the parent cursor unchanged. Use the
[sequences and events guide](sequences.md) for runnable comparisons and editable pauses.

## Return to a saved pose

Capture a pose before a temporary change. `restore` returns to that captured state.
`reset` returns to the constructor values. Here the saved position is left of center,
while the constructor position is the center.

<GuideDemo name="animation-snapshot" caption="The circle grows and moves, restores its left pose, then resets to the center.">

<!-- focus: animation-snapshot -->
```lua
ball.position.x:set(-256)
local pose = ball:snapshot()

ball:to { position = { x = 256 }, radius = 64 }:play(1)
ball:restore(pose):play(1)
ball:reset():play(1)
```

<template #source>

<!-- demo: animation-snapshot -->
<<< ./projects/animation/scenes/02-animation-snapshot.lua

</template>

</GuideDemo>

## Snapshots and time

Snapshots include animatable fields, dynamic properties, and uniforms. They belong to
the object that created them. `save()` pushes a snapshot. `restore():play(1)` pops it.
Property snapshots do not replace [simulation state](simulation.md).

## Timeline events and independent tracks

Use `k.event("Reading")` for a pause you can adjust in Timeline, or `k.wait(seconds)`
for a fixed pause. See [editable pauses](sequences.md#editable-pauses).

`k.track(initial, id)` creates an independent animated property. Use it for a shared
parameter that drives several [signals](signals.md), without adding a visible object.
The optional ID preserves its identity across reloads.
