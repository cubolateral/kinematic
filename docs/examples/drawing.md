# Procedural drawing

Draw paths, curves, and gradients in local coordinates.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/drawing.png')" style="width:100%" :src="withBase('/previews/drawing.mp4')" aria-label="Procedural drawing preview"></video>

[Guide](../guide/drawing.md) · [API reference](../api/Draw2D.md)

## What to look for

A Lua object draws a ribbon. Its transform belongs to the object, and a snapshot
restores its values without changing the drawing code.

### `scenes/intro.lua`

<<< ../../examples/lua/drawing/scenes/intro.lua

## Try it yourself

Change the control points and update bounds to include the full shape.
