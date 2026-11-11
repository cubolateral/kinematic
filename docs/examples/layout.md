# Rectangles and groups

Animate size and position while keeping a shared baseline.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/layout.png')" style="width:100%" :src="withBase('/previews/layout.mp4')" aria-label="Rectangles and groups preview"></video>

[Guide](../guide/objects.md) · [API reference](../api/Panel.md)

## What to look for

Rectangles are attached to a group. Their height and position change together to keep
their bottom edges aligned. Each animation starts with a short delay.

### `scenes/intro.lua`

<<< ../../examples/lua/layout/scenes/intro.lua

## Try it yourself

Change the heights while preserving the shared baseline. Then compare the result with a
layout container.
