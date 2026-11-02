# Text

Use `text_2d` and `text_3d` to add ordinary text to a scene. These are graphic objects,
not editor input fields. Use [LaTeX](latex.md) for mathematical expressions and
[Code](code.md) for syntax-highlighted source and range-based edits.

## Content, fonts, and size

`text` contains the string and can include line breaks. `size` sets the type size in
local units. `align` ranges from -1 (left) to 1 (right), with 0 centered. `thickness`
expands glyph strokes.

Text objects use Hack by default in this checkout. To make your project independent of
build paths, load a project TTF or OTF with `k.font(path)` and pass it as `font=...`.
The example includes the font and its licenses. Include both the file and its license
when you distribute the project. The font resource cannot be animated.

## Transitions

`text:morph(new_text,group?)` moves matching glyphs and fades in or out the rest.
`group=true` matches words and symbols as groups. The default is false. Call `play` to
schedule the transition. Available methods depend on the object type:

| Type | Transitions |
| --- | --- |
| Text2D | morph and fade. |
| Text3D | morph. |

`fade` on Text2D crossfades the entire text block. Do not assume Text3D has the same
methods just because the objects share content properties.

<<< ../../examples/lua/text/scenes/intro.lua

The [text project](../examples/text.md) demonstrates the text object and its
transitions. The first morph preserves matching glyphs, while the following fade
replaces the whole block.

## Composition and 3D

Bounds depend on the text and font. Use anchors in parent coordinates and leave room for
both strings during a transition. Line breaks change the bounding box, so `position`
does not mean the position of the first character.

Text3D uses `vec3`, quaternions, and material properties. Apparent size depends on the
camera. For text that stays the same size in the frame, use a 2D object in the
composition. For a panel in space, use Text3D or project a 2D texture into 3D.

See [Text2D](../api/Text2D.md), [Text2DOptions](../api/Text2DOptions.md),
[Text3D](../api/Text3D.md), and [Text3DOptions](../api/Text3DOptions.md). Render the
project to check the font, glyph matching, and readability at the final resolution.
