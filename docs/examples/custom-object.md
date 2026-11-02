# Custom objects

Define custom properties, methods, bounds, and 2D or 3D drawing.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/custom-object.png')" style="width:100%" :src="withBase('/previews/custom-object.mp4')" aria-label="Custom objects preview"></video>

[Guide](../guide/custom-objects.md) · [API reference](../api/ObjectOptions.md)

## What to look for

The first scene creates two bars from one Lua object definition with independent
parameters. The second draws a custom mesh in three instances.

### `scenes/intro.lua`

<<< ../../examples/lua/custom-object/scenes/intro.lua

### `scenes/mesh.lua`

<<< ../../examples/lua/custom-object/scenes/mesh.lua

## Try it yourself

Add a dynamic property to the bar and use it in both `draw` and `bounds`. Then change
the mesh transforms.
