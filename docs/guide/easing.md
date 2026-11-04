---
pageClass: guide-page
---

# Easing

Duration says when an animation ends. Easing maps its progress to a value, changing
speed within that duration. Compare curves over the same distance and time.

## Constant speed and smooth starts

<GuideDemo name="easing-compare" caption="Top to bottom: linear, cubic in, cubic out, and cubic in out.">

<!-- demo: easing-compare -->
<<< ./projects/easing/scenes/01-easing-compare.lua

</GuideDemo>

All four circles start and finish together. Linear covers equal distances in equal
intervals. `_in` begins slowly, `_out` ends slowly, and `_in_out` does both. The default
is `"cubic_in_out"`. Try changing only one duration to see timing and easing separately.

```text
Progress in time:  0%          50%         100%
Linear distance:   0%          50%         100%
Cubic in:          0%          12.5%       100%
Cubic out:         0%          87.5%       100%
```

## Overshoot and bounce

Use back for a small overshoot, elastic for oscillation, and bounce for repeated impacts.
These curves draw attention. Compare them before using them for a quiet transition.

<GuideDemo name="easing-overshoot" caption="Top: back overshoots. Middle: elastic oscillates. Bottom: bounce rebounds.">

<!-- demo: easing-overshoot -->
<<< ./projects/easing/scenes/02-easing-overshoot.lua

</GuideDemo>

Leave room beyond the destination for overshoot. Try `"sine_in_out"` for a smoother,
quieter movement. Other families are `quad`, `quart`, `quint`, `expo`, and `circ`.
Each family provides `_in`, `_out`, and `_in_out`. Use the exact
[API spellings](../api/Aliases.md#property-kinematic-easing).

Continue with [sequences and events](sequences.md).
