# Canvases and projections

Display a live 3D texture through a 2D projection.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/composition.png')" style="width:100%" :src="withBase('/previews/composition.mp4')" aria-label="Canvases and projections preview"></video>

[Guide](../guide/canvases.md) · [API reference](../api/Projection2D.md)

## What to look for

The sphere and cube belong to the source canvas. A projection displays its live texture
in the destination without copying pixels in Lua.

### `scenes/intro.lua`

<<< ../../examples/lua/composition/scenes/intro.lua

## Try it yourself

Change the texture resolution and projection size independently.
