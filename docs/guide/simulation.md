# Simulation

## Objects with state

Simulations belong to objects declared with `k.object`. The [custom objects
guide](custom-objects.md) covers definitions, instances, components, properties,
methods, bounds, and drawing. The [signals guide](signals.md) covers temporary changes
that do not accumulate state. This page focuses on fixed steps, dependencies, and
seeking through time.

## State and fixed steps

An object with `state` and `update(state,frame,inputs)` keeps state per instance. The
runtime updates it at a fixed step derived from the project FPS, including when you
seek. Each instance gets its own copy of the initial state, and `state()` returns
another copy for inspection.

The [reproducible walk project](../examples/simulation.md) accumulates a path using a
random generator stored in state:

<<< ../../examples/lua/simulation/scenes/intro.lua

Do not use draw time or monitor refresh rate to update physics. Drawing reads copies of
evaluated properties and state, and should produce the same result for the same
evaluation.

## Inputs and dependencies

Declare `inputs={speed="speed"}` for a property on the same object,
`inputs={speed=other.speed}` for a property handle, or `inputs={source=other}` for
another simulation's copied state. Use `object:inputs({...})` to configure inputs after
construction. Audio players can also be inputs.

Dependencies are sampled at each fixed step, not only at the final seek time.
Dependencies update before their dependents, and cycles are rejected. Omitted inputs
conservatively use the object's own properties. `inputs={}` declares no dependencies.

## Checkpoints and mutations

`auto_update:set(false)` disables automatic updates. `update()` schedules one explicit
step, `checkpoint()` schedules a snapshot to speed up reconstruction, and
`mutate(function(state) ... end)` schedules a state change. Each operation uses the
current timeline time.

When seeking backward, the engine restores a compatible checkpoint and replays the steps
it needs. Changes to logic, initial state, seed, FPS, lifetime, inputs, or operations
invalidate the affected history and its dependents. Visual-only changes preserve
compatible checkpoints. Current invalidation replays an affected instance from the
beginning. It does not update each segment incrementally.

## Callback restrictions

Update and mutate run in an isolated, read-only environment. They cannot capture mutable
external values. Put mutable data in `state` and external relationships in `inputs`.
`require` accepts pure helpers in this restricted environment, and exported functions
follow the same closure validation. Cycles, mutable exports, and writes to globals are
rejected.

State can contain finite scalars, simple tables with string or integer keys, vector and
color values, quaternions, quads, Random values, and immutable image, font, mesh, and
matrix handles. Copies preserve random state. Cyclic or shared tables, metatables,
functions, and unsupported userdata are errors. The maximum nesting depth is 64.

## Reproducible randomness

Use `k.random(seed)` in state. `number` returns a value in [0,1). `range` and
`range_integer` use an exclusive upper bound. Other methods include `chance`, `normal`,
`choose`, `choose_multiple`, and `shuffle`. `fork(key)` derives an independent generator
without consuming the parent. See [Random](../api/Random.md).

`math.random` is unavailable in simulation callbacks. Construction uses a fixed initial
seed. Prefer ordered arrays when iteration order affects numeric results.
