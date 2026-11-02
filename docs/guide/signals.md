# Signals

A signal adjusts a frame after its animation tracks have been evaluated. Use one for
visual relationships, such as a position driven by an animated phase or a property
derived from another value. Signals do not accumulate state across frames and are not a
replacement for simulation.

## Create and stop a signal

`object:signal(callback)` receives the object and the frame. `property:signal(callback)`
receives the property handle and the frame. Both return a [Signal](../api/Signal.md)
with a `stop()` method. Creating and stopping a signal are scheduled at the current
scene time.

<<< ../../examples/lua/signals/scenes/intro.lua

The example uses an independent `s:track` for its phase. The track has a regular
animation, and the signal converts its value into a position. The circle is a built-in
object with a transparent fill and an outline. The signal stops when the orbit is
complete.

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
the end of the relationship, as in the orbit example.

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
