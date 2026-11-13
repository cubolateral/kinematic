---
editLink: false
outline: 2
---

# Animation

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

One interval or a list of intervals. Lists may be nested.
Vector, color, quaternion, or quad value.
Reusable animation exposing :delay and :play. Preparing it leaves the timeline unchanged.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Animation.html">Animation</a><span class="api-punctuation">:</span><span class="api-function">delay</span><span class="api-punctuation">(</span><span class="api-parameter">seconds</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#animation-delay-seconds}

<div class="api-description">

Returns a copy with an initial delay for all tracks. Preparing it leaves the timeline unchanged.
Seconds must be finite and nonnegative. Replaces any previous delay; each play includes the delay in its duration.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Animation.html">Animation</a><span class="api-punctuation">:</span><span class="api-function">play</span><span class="api-punctuation">(</span><span class="api-parameter">duration</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">easing</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">Easing</span><span class="api-punctuation">)</span></code> {#animation-play-duration-easing}

<div class="api-description">

Schedules an independent execution at the current scene time, including its configured delay.
Duration is in seconds and defaults to zero for an instantaneous change. Easing accepts only snake_case strings and defaults to "cubic_in_out".

</div>

</section>

</div>

:::
