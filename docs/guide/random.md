---
pageClass: guide-page
---

# Random

Randomness can create variation while remaining reproducible. A seed initializes an
ordered sequence of draws. The same seed and the same calls produce the same values.

## Compare seeds

<GuideDemo name="random-seeds" caption="The first two rows match exactly. A different seed changes the last row." still>

<!-- demo: random-seeds -->
<<< ./projects/random/scenes/01-random-seeds.lua

</GuideDemo>

`range(10, 30)` includes 10 and excludes 30. `range_integer` also excludes its upper
bound. Use `range_integer(1, #items + 1)` for a valid array index. `number()` returns
[0, 1), `integer()` returns an unsigned 32-bit integer, and `chance(0.25)` succeeds
with probability one quarter. Inserting a draw changes subsequent results.

## Uniform and normal distributions

<GuideDemo name="random-distribution" caption="Uniform samples spread across the top band. Normal samples gather near the center below." still>

<!-- demo: random-distribution -->
<<< ./projects/random/scenes/02-random-distribution.lua

</GuideDemo>

A normal distribution is not confined to a fixed interval. Increasing deviation spreads
the lower cloud. Try `normal(120, 40)` to shift its center and narrow it.

## Choose, weight, and shuffle

```lua
local random = k.random(42)
local palette = {"#FFFFFF", "#40FF80", "#FF40BF"}
local color = random:choose(palette, {6, 3, 1})
local pair = random:choose_multiple(palette, 2)
local reordered = random:shuffle(palette)
```

The weights make white more likely. They do not guarantee exact proportions in a small
sample. `choose_multiple` samples without replacement. `shuffle` permutes the input array
in place and returns that same table. Copy the table first if you need to preserve its
original order. Use the chosen color in the circles above to compare a weighted palette.

## Independent streams and simulation

`random:fork(1)` derives an independent generator without consuming the parent. Assign
stable integer keys to independent effects so changing one effect's draw count does not
shift the other. A repeated key derives the same initial stream.

During scene construction, create the generator locally. For an accumulated random walk,
store it in simulation `state` so checkpoints preserve the sequence. Do not construct a
new generator on every update or draw random values while drawing. `math.random` is
unavailable in simulation callbacks. The [simulation guide](simulation.md) explains
seeking and the [Random API](../api/Random.md) lists the methods.
