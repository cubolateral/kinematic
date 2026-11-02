# Signals

Derive an object’s position from an independent animated track.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/signals.png')" style="width:100%" :src="withBase('/previews/signals.mp4')" aria-label="Signals preview"></video>

[Guide](../guide/signals.md) · [API reference](../api/Signal.md)

## What to look for

An animated phase drives a circle’s position through a signal. The callback stops after
one orbit, without accumulating simulation state.

### `scenes/intro.lua`

<<< ../../examples/lua/signals/scenes/intro.lua

## Try it yourself

Try a different easing curve and seek directly to different times.
