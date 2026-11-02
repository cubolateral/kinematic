# Audio

Schedule sound effects with movement and control stereo pan.

<script setup>
import { withBase } from 'vitepress'
</script>

<video controls preload="metadata" :poster="withBase('/previews/audio.png')" style="width:100%" :src="withBase('/previews/audio.mp4')" aria-label="Audio preview"></video>

[Guide](../guide/audio.md) · [API reference](../api/Audio.md)

## What to look for

The same synthesized resource plays for three impacts, with pan following each position.
A WAV playback ends the demonstration.

### `scenes/intro.lua`

<<< ../../examples/lua/audio/scenes/intro.lua

## Try it yourself

Change the preset and pan. Keep playback at the moment of impact.
