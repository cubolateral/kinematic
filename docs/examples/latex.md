# LaTeX

Morph between mathematical expressions and fade to a new formula.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/latex.png')" style="width:100%" :src="withBase('/previews/latex.mp4')" aria-label="LaTeX preview"></video>

[Guide](../guide/latex.md) · [API reference](../api/Latex2D.md)

## What to look for

A centered formula demonstrates glyph matching, block transitions, and color animation.

### `scenes/intro.lua`

<<< ../../examples/lua/latex/scenes/intro.lua

## Try it yourself

Compare morph with `group=true` and the default `false`.
