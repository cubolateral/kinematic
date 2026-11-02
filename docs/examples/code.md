# Code

Search, highlight, select, and animate edits to source code.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/code.png')" style="width:100%" :src="withBase('/previews/code.mp4')" aria-label="Code preview"></video>

[Guide](../guide/code.md) · [API reference](../api/Code.md)

## What to look for

The Code object is the subject of the example. Searches return ranges, edit methods
change the text, and morph rearranges the final block.

### `scenes/intro.lua`

<<< ../../examples/lua/code/scenes/intro.lua

## Try it yourself

Change the literal search and calculate the ranges again after editing.

The project bundles the Hack font for consistent rendering. Its license notices are kept
beside the font in `assets/`.
