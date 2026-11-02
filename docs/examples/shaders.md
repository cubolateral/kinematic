# Shaders

Animate uniforms in image and mesh shaders.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/shaders.png')" style="width:100%" :src="withBase('/previews/shaders.mp4')" aria-label="Shaders preview"></video>

[Guide](../guide/shaders.md) · [API reference](../api/Shader.md)

## What to look for

The rectangle on the left uses image post-processing. The cube on the right uses a mesh
fragment shader. Both animate `pulse`.

### `scenes/intro.lua`

<<< ../../examples/lua/shaders/scenes/intro.lua

## Try it yourself

Change the stripe frequency and render the project to inspect both GLSL contracts.
