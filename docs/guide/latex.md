---
pageClass: guide-page
---

# LaTeX

Use `latex_2d` for mathematical notation. Kinematic builds formula geometry with RaTeX
and the bundled KaTeX fonts. A separate TeX installation is not needed. Start with a
formula, then use `morph` to develop it or `fade` to introduce a different expression.

## Create a formula

Pass the expression in `text`, without `$` delimiters. Lua long strings (`[[...]]`)
keep backslashes intact, so `\frac` needs only one backslash in the source.

<GuideDemo name="latex-formula" caption="The Pythagorean theorem rendered from the expression below." still>

<!-- demo: latex-formula -->
<<< ./projects/latex/scenes/01-latex-formula.lua

</GuideDemo>

`size` controls the typography in local units. Position, opacity, and anchors work as
for other 2D objects. Fractions and exponents can extend above and below the baseline.

## Compare morph modes

Both videos change mean velocity into mean acceleration. Only the grouping flag changes.
Compare the paths of matching glyphs with the behavior of complete layout groups.

### Match glyphs: group = false

<GuideDemo name="latex-morph-false" caption="Individual matching glyphs move into their new positions.">

<!-- focus: latex-morph-false -->
```lua
formula:morph(
    [[\text{mean acceleration} = \frac{\Delta\text{velocity}}{\Delta\text{time}}]],
    false):play(1)
```

<template #source>

<!-- demo: latex-morph-false -->
<<< ./projects/latex/scenes/02-latex-morph-false.lua

</template>

</GuideDemo>

### Match groups: group = true

<GuideDemo name="latex-morph-true" caption="Matching groups move together, while changed groups fade.">

<!-- focus: latex-morph-true -->
```lua
formula:morph(
    [[\text{mean acceleration} = \frac{\Delta\text{velocity}}{\Delta\text{time}}]],
    true):play(1)
```

<template #source>

<!-- demo: latex-morph-true -->
<<< ./projects/latex/scenes/03-latex-morph-true.lua

</template>

</GuideDemo>

With `false`, matching symbols can travel separately between the expressions. With `true`,
the renderer matches whole layout groups, including fraction and script structure.
A changed group can fade instead of separating into individual glyphs.

## Fade to a new formula

Use `fade` when two expressions should replace one another as complete blocks. Their
opacities overlap during the transition. Matching glyphs do not move between formulas.

<GuideDemo name="latex-fade" caption="One equation crossfades into another without matching their terms.">

<!-- focus: latex-fade -->
```lua
formula:fade([[x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}]]):play(1)
```

<template #source>

<!-- demo: latex-fade -->
<<< ./projects/latex/scenes/04-latex-fade.lua

</template>

</GuideDemo>

Both transitions return animation definitions. Use `play(duration, easing)` to schedule
them, just as with property animations. Use ordinary `to` animations for position,
color, or opacity rather than changing the expression to create those effects.

## Supported syntax

The renderer accepts a subset of mathematical display-style syntax, not full LaTeX
documents. Do not include preambles, `\documentclass`, or package-loading commands.
Installing TeX does not extend the supported syntax.

If a formula fails, first check balanced braces and backslashes. A quoted Lua string
needs `"\\frac{a}{b}"`. A long string needs `[[\frac{a}{b}]]`. Try the shortest failing
expression to isolate an unsupported command. Check every destination formula too,
since transitions also need to build geometry.

## Formulas in 3D

The 3D mesh follows the same outlines as the 2D version, with finely sampled curves
and extrusion depth.

Use `latex_3d` to place a formula in space. A full turn makes the perspective visible
from every angle. Its size uses spatial units, and `albedo` supplies its material color. An unlit material keeps it readable without adding lights.

<GuideDemo name="latex-spatial" caption="The formula completes a full turn in a continuous five-second loop.">

<!-- demo: latex-spatial -->
<<< ./projects/latex/scenes/05-latex-spatial.lua

</GuideDemo>

Camera distance changes the apparent size. Keep labels in 2D when they should stay
fixed on screen. Position uses `vec3`, and rotation uses a quaternion.

Latex2D and Latex3D support both `morph` and `fade`. See
[Latex2D](../api/Latex2D.md) and [Latex3D](../api/Latex3D.md) for their options.
