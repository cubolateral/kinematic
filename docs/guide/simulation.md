---
pageClass: guide-page
---

# Simulation

Ordinary animations are functions of time: Kinematic can evaluate a property at two
seconds without playing the preceding frames. A simulation is different. Its next
state depends on what happened before, such as velocity, collisions, or an accumulated
trail. It stores that history and replays fixed steps when you seek.

Use [animation](animation.md) for known destinations and [signals](signals.md) for
values calculated directly from time. Use simulation in a [custom object](custom-objects.md)
when that calculation needs the previous state.

## Integrate speed into position

This particle adds `speed × dt` to its position on every step. The speed is an animated
property, sampled through `inputs`. Its trail records where the particle has been.
Drawing only reads those points.

<GuideDemo name="simulation-step" caption="The trail accumulates while the particle slows from 160 to 60 units per second.">

<!-- focus: simulation-step -->
```lua
on_update = function(self, frame, inputs)
    self.state.x = self.state.x + inputs.speed * frame.dt
    self.state.trail[#self.state.trail + 1] = self.state.x
    if #self.state.trail > 120 then
        table.remove(self.state.trail, 1)
    end
end,
```

<template #source>

<!-- demo: simulation-step -->
<<< ./projects/simulation/scenes/01-simulation-step.lua

</template>

</GuideDemo>

The step duration comes from the project FPS, not the monitor refresh rate. At one
second, the particle has traveled about 160 units. During the next second, each step
reads the changing speed. It does not apply the final speed to the entire history.
The example keeps at most 120 trail points so the state does not grow indefinitely.

Expand **Complete scene** to see the property, dependency, bounds, and drawing callback.
Seek to two seconds, back to zero, and then to two seconds again: the trail should agree.

## Declare dependencies

`inputs` tells the engine which values influence the next state. Use local property
names, property handles, or another simulation's copied state:

```lua
inputs = {speed = "speed"},
```

After creating an instance, `ball:inputs({speed = other.speed})` can bind it to another
object's speed. Dependencies are evaluated before their dependents. Cycles are rejected.
Omitting `inputs` conservatively includes the object's own properties. `inputs = {}`
declares no property dependencies. Own properties are also available as evaluated values
on `self` when included in the declared inputs. With explicit inputs, declare every own
property the update reads. State remains mutable through `self.state`.

Keep changing data in `state`. A mutable variable captured outside `on_update` would not
be restored when seeking, so simulation callbacks reject such captures.

## Schedule a state change

Use `mutate` for an event that changes state at a known time. Here the particle returns
to the left and clears its trail after one second. The operation is part of the
timeline, so seeking across it reproduces the same reset.

<GuideDemo name="simulation-mutate" caption="At one second, the position and trail reset. Integration then continues.">

<!-- focus: simulation-mutate -->
```lua
k.wait(1)
ball:mutate(function(state)
    state.x = -240
    state.trail = {}
end)
k.wait(1)
```

<template #source>

<!-- demo: simulation-mutate -->
<<< ./projects/simulation/scenes/02-simulation-mutate.lua

</template>

</GuideDemo>

`ball:state()` returns a copy for inspection. Editing that copy does not change the
simulation. Property `reset` and snapshots also do not reset simulated state. Use a
scheduled mutation for that behavior.

## Checkpoints and seeking

`ball:checkpoint()` schedules a saved state at the current time. It can reduce the
work needed to reconstruct later frames. It does not create a visible effect. On a
backward seek, the engine restores a compatible checkpoint and replays the needed steps.

Changes to logic, initial state, FPS, lifetime, dependencies, or scheduled operations
invalidate affected history. Visual properties that are not simulation inputs can
preserve compatible checkpoints. Keep dependencies explicit for expensive simulations.

For manual stepping, set `ball.auto_update:set(false)` and call `ball:update()` at
the timeline times where a step should occur. Ordinary simulations can leave automatic
updates enabled.

## Reproducible state

State supports finite scalar values, simple tables, supported vectors and colors,
Random values, and immutable resource handles. Functions, metatables, cyclic or shared
tables, and unsupported userdata are rejected. Each instance receives its own copy.

For randomness, put `k.random(seed)` in state and draw values inside `on_update`. Checkpoints
then preserve the random sequence. Do not use `math.random`, sample random values in
`on_draw`, or recreate the generator at every step. See [Random](random.md).

Update and mutate run in an isolated environment. Pure helper modules are allowed.
Mutable exports and writes to globals are not. If a result changes when seeking, check
state, inputs, and randomness before adding checkpoints. Checkpoints accelerate correct
reconstruction. They cannot repair undeclared dependencies.
