# Text

Animate changes to text content.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/text.png')" style="width:100%" :src="withBase('/previews/text.mp4')" aria-label="Text preview"></video>

[Guide](../guide/text.md) · [API reference](../api/Text2D.md)

## What to look for

The Text2D object demonstrates morph and fade transitions. The phrases are part of the
text object.

### `scenes/intro.lua`

<<< ../../examples/lua/text/scenes/intro.lua

## Try it yourself

Change the phrases and compare glyph-level and word-level matching.

The project bundles the Hack font for consistent rendering. Its license notices are kept
beside the font in `assets/`.
