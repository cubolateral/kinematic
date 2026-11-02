# LaTeX

`latex_2d` and `latex_3d` turn mathematical expressions into geometry. They use RaTeX
and KaTeX fonts included with Kinematic. TeX is not run on your system. Use these
objects for equations and mathematical symbols, [Text](text.md) for prose, and
[Code](code.md) for source code.

## Create a formula

Pass an expression in the `text` field without `$` delimiters. `size` sets its size in
local units, and `thickness` expands the strokes. The 2D object has fill and outline
properties. The 3D object uses material properties.

```lua
local formula = s:latex_2d {
    text = [[\frac{a}{b} + \sqrt{x}]],
    size = 64,
    fill = k.color("#FFFFFF"),
}
s:world_2d():add(formula)
```

Lua's long string syntax `[[...]]` avoids doubling backslashes. With a quoted string,
use `"\\frac{a}{b}"` for the same input. A formula is a regular scene object. Attach it
to a world or group, then use its position, scale, opacity, bounds, and anchors.

## Syntax and compatibility

The renderer supports a subset of mathematical display-style syntax. Do not pass
`\\documentclass`, preambles, auxiliary files, or TeX package commands. Installing TeX
does not add support. An invalid or unsupported expression can fail when its geometry is
requested, including when it is used later in a transition.

Check expressions in the Kinematic version you plan to use. `--check` verifies
construction and evaluation. Render the scene to inspect geometry and appearance.
Support for an individual command does not mean full LaTeX documents are compatible.

## Morph and fade

`formula:morph(new_text, group?)` prepares a transition that moves matching glyphs and
fades in or out the rest. With `group=true`, words and symbols are matched as groups
instead of individual glyphs. The default is false.

`formula:fade(new_text)` prepares a crossfade of the full expression. Both methods
return an animation definition. Call `:play(duration,easing)` to schedule it. Creating
the definition does not change the formula or timeline.

<<< ../../examples/lua/latex/scenes/intro.lua

The [LaTeX project](../examples/latex.md) keeps the formula centered, changes terms with
morph, and then replaces the block with fade. Color and opacity use regular property
animations.

## Position and bounds

Bounds depend on the expression. Fractions, radicals, and exponents change its height
and width. Use `bounds`, `box_size`, and anchors to align a formula with other objects.
During a transition, allow room for both the old and new expression. Do not size the
layout for only the initial text.

Font size and scale are separate controls. `size` affects typography geometry, while
`scale` transforms the object and its descendants. Avoid compensating for a distant
camera with very large text without checking the final resolution and readability.

## Formulas in 3D

`latex_3d` uses `vec3` position and scale, a quaternion, and material properties. Attach
it to the 3D world or a 3D canvas. Its apparent size depends on the camera:

```lua
local formula = s:latex_3d {
    text = [[e^{i\pi}+1=0]],
    size = 1,
    albedo = k.color("#FFFFFF"),
    unlit = true,
}
s:world_3d():add(formula)
s:view("3d")

formula:morph("a^2+b^2=c^2"):play(1)
```

LaTeX2D and LaTeX3D provide morph and fade. Do not assume other text objects share this
list. Text3D, for example, has morph but no fade method. The [generated API
reference](../api/Latex2D.md) lists each type separately.

## Performance and troubleshooting

The renderer reuses mathematical geometry. A sequence of large or very different
expressions increases preparation and render time. Test with a shorter expression to
find a problematic command. Check escaped backslashes, balanced braces, and the absence
of document delimiters.

See [Latex2D](../api/Latex2D.md), [Latex2DOptions](../api/Latex2DOptions.md),
[Latex3D](../api/Latex3D.md), and [Latex3DOptions](../api/Latex3DOptions.md).
