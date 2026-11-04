---
pageClass: guide-page
---

# Signals

A signal adjusts a frame after its animation tracks have been evaluated. Use one for
visual relationships, such as a position driven by an animated phase or a property
derived from another value. Signals do not accumulate state across frames and are not a
replacement for simulation.

## Derive a follower position

Use a signal when one object's appearance depends on another object's evaluated value.
The leader has a regular track. The follower calculates its position each frame.

<GuideDemo name="signal-follow" caption="The blue follower stays 128 units below the animated white leader.">

<!-- demo: signal-follow -->
<<< ./projects/signals/scenes/01-signal-follow.lua

</GuideDemo>

The callback reads the evaluated leader track and writes a temporary follower position.
It does not integrate velocity. Try doubling `position.x` in the callback to make the
follower travel twice as far. When multiple signals depend on one another, avoid relying
on callback order. Prefer reading the shared driving track.

## Create and stop a signal

`object:signal(callback)` receives the object and the frame. `property:signal(callback)`
receives the property handle and the frame. Both return a [Signal](../api/Signal.md)
with a `stop()` method. Creating and stopping a signal are scheduled at the current
scene time.

## Frame timing

`frame.time` is the evaluated time in seconds, `frame.index` is the frame number, and
`frame.dt` is the frame duration. Seeking can call the callback without drawing earlier
frames. Derive values from time and evaluated properties instead of incrementing
captured variables.

```lua
local signal = ball:signal(function(object, frame)
    object.position:set(k.vec2(math.sin(frame.time * 2) * 100, 0))
end)

s:wait(2)
signal:stop()
```

An object's callback runs only while the object is active and within the signal's
lifetime. `remove` ends the object's presence over time. Seeking backward restores
earlier intervals.

## Temporary writes

Tracks are evaluated before signals. A `set` call in the callback replaces the visible
value for that frame only. The engine restores the base value before the next
evaluation. If a callback fails, its partial writes are not applied.

Stopping a signal does not turn its last calculated value into a keyframe. The property
returns to its track or construction value. To avoid a jump, make the base value match
the end of the relationship, at the scheduled stop time.

## Stop a derived motion

A signal's last write does not become an animation keyframe. Stop it deliberately to
see the track resume.

<GuideDemo name="signal-stop" caption="Two seconds of circular motion end with a jump to the base position at the origin.">

<!-- focus: signal-stop -->
```lua
local signal = ball:signal(function(object, frame)
    object.position:set(k.vec2(math.cos(frame.time * 5), math.sin(frame.time * 5)) * 128)
end)

s:wait(2)
signal:stop()
```

<template #source>

<!-- demo: signal-stop -->
<<< ./projects/signals/scenes/02-signal-stop.lua

</template>

</GuideDemo>

The jump is expected because the underlying position is still zero. To preserve the
endpoint, schedule a base position matching `k.vec2(math.cos(10), math.sin(10)) * 128` when stopping. Try
changing the stop time and predict the endpoint before seeking there.

## Callback restrictions

A callback can read time, values, and compatible resources, then write properties for
the current frame. It cannot create or attach objects, schedule work with `play`, or
trigger audio. Do not retain a drawing context for use here. For accumulated paths or
collision events, use [simulation](simulation.md) and inputs.

## Choose the right tool

| Need | Use |
| --- | --- |
| Move between values over a known duration. | `to`/`by`/`from` and `play`. |
| Calculate a position from a phase or time. | Signal. |
| Integrate velocity or accumulate points. | State and update in a custom object. |
| Draw an already evaluated state. | Draw callback. |

Signals can work with full vector properties or individual channels. See
[Property](../api/Property.md), [Vec2Property](../api/Vec2Property.md), and the
[runnable example](../examples/signals.md).
