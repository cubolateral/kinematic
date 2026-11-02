# Simulation

Explore fixed-step state, checkpoints, and reproducible randomness.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/simulation.png')" style="width:100%" :src="withBase('/previews/simulation.mp4')" aria-label="Simulation preview"></video>

[Guide](../guide/simulation.md) · [API reference](../api/Random.md)

## What to look for

A path accumulates points in state using a fixed seed. Drawing reads the result, and a
checkpoint divides the reconstruction.

### `scenes/intro.lua`

<<< ../../examples/lua/simulation/scenes/intro.lua

## Try it yourself

Change the seed, then seek backward and confirm that the same seed reproduces the same
path.
