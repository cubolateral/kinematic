---
pageClass: guide-page
---

# Text

Use `text_2d` for titles, labels, and paragraphs. Start with the text and its size.
Then choose whether a content change should move matching letters or replace the whole
block. Use [LaTeX](latex.md) for formulas and [Code](code.md) for highlighted source.

## Create a title

Attach the title to the 2D world, just like a shape. `size` uses local canvas units.
`fill` controls the glyph color. The built-in font is Hack.

<GuideDemo name="text-title" caption="A centered title in the built-in font." still>

<!-- demo: text-title -->
<<< ./projects/text/scenes/01-text-title.lua

</GuideDemo>

## Compare morph modes

Both videos begin with `Hello, world!` and end with `Hello, wonderful world!`.
Only the grouping flag changes. `play` schedules the transition over one second.

### Match glyphs: group = false

<GuideDemo name="text-morph-false" caption="Individual matching glyphs move into their new positions.">

<!-- focus: text-morph-false -->
```lua
title:morph("Hello, wonderful world!"):play(1)
```

<template #source>

<!-- demo: text-morph-false -->
<<< ./projects/text/scenes/02-text-morph-false.lua

</template>

</GuideDemo>

### Match groups: group = true

<GuideDemo name="text-morph-true" caption="Matching groups move together, while changed groups fade.">

<!-- focus: text-morph-true -->
```lua
title:morph("Hello, wonderful world!", true):play(1)
```

<template #source>

<!-- demo: text-morph-true -->
<<< ./projects/text/scenes/03-text-morph-true.lua

</template>

</GuideDemo>

With `false`, matching letters move individually into their new positions. With `true`,
the inserted `wonderful` appears as a group while matching words stay together. The matching greeting
stays recognizable in both versions.

## Fade between titles

Use `fade` for a new heading that does not need a letter-by-letter relationship. It
crossfades the complete old and new blocks, including any letters they share.

<GuideDemo name="text-fade" caption="The greeting crossfades to “This is a faded text!”.">

<!-- focus: text-fade -->
```lua
title:fade("This is a faded text!"):play(1)
```

<template #source>

<!-- demo: text-fade -->
<<< ./projects/text/scenes/04-text-fade.lua

</template>

</GuideDemo>

## Align multiple lines

Use `\n` for a line break. `align` positions lines within their text block: `-1` is
left, `0` is center, and `1` is right. It does not align the block with other objects.
Use [anchors](coordinates.md#bounds-and-anchors-use-different-spaces) for that.

<GuideDemo name="text-alignment" caption="Left, middle, and right blocks show the three line alignments." still>

<!-- demo: text-alignment -->
<<< ./projects/text/scenes/05-text-alignment.lua

</GuideDemo>

## Use a project font

Put a TTF or OTF file in your assets and load it once. Paths are relative to the project
root. Include its license when distributing the project.

```lua
local heading_font = k.font("assets/heading.ttf")
local title = s:text_2d {
    text = "A new chapter",
    font = heading_font,
    size = 56,
    fill = k.color("#FFFFFF"),
}
s:world_2d():add(title)
```

The font is a resource, not an animated channel. Changing the text, size, or font can
change bounds. `thickness` expands glyph strokes. Check alignment after such changes.

## Text in 3D

Use `text_3d` to place text in space. A full turn makes the perspective visible
from every angle. Its size uses spatial units, and `albedo` supplies its material color. An unlit material keeps it readable without adding lights.

<GuideDemo name="text-spatial" caption="The text completes a full turn in a continuous five-second loop.">

<!-- demo: text-spatial -->
<<< ./projects/text/scenes/06-text-spatial.lua

</GuideDemo>

Camera distance changes the apparent size. Keep labels in 2D when they should stay
fixed on screen. Position uses `vec3`, and rotation uses a quaternion.

Text2D supports `morph` and `fade`. Text3D supports `morph` only.
See [Text2D](../api/Text2D.md) and [Text3D](../api/Text3D.md) for their options.
