---
editLink: false
outline: 2
---

# Audio

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Scene-local audio player with independent playback defaults.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Audio.html">Audio</a><span class="api-punctuation">:</span><span class="api-function">play</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioPlaybackOptions.html">AudioPlaybackOptions</a><span class="api-punctuation">)</span></code> {#audio-play-options}

<div class="api-description">

Schedules an occurrence without advancing the animator; overrides affect only this occurrence.
In simulation updates, use a declared audio input; the occurrence uses the step time.

</div>

</section>

</div>

:::
