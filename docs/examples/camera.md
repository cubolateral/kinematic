# Cameras

Explore 3D framing and animate a 2D camera.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/camera.png')" style="width:100%" :src="withBase('/previews/camera.mp4')" aria-label="Cameras preview"></video>

[Guide](../guide/cameras.md) · [API reference](../api/Canvas3D.md)

## What to look for

The first scene uses `focus_camera` to frame a group of cubes. The second animates the
2D camera position, zoom, and rotation.

### `scenes/intro.lua`

<<< ../../examples/lua/camera/scenes/intro.lua

### `scenes/2d.lua`

<<< ../../examples/lua/camera/scenes/2d.lua

## Try it yourself

Compare perspective and orthographic views. Then keep the objects still and animate only
the camera.
