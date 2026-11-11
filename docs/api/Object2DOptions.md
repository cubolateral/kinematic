---
editLink: false
outline: 2
---

# Object2DOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Scripted 2D callbacks; additional functions are ordinary instance methods or helpers.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="ObjectOptions.html">ObjectOptions</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">on_draw</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Script2DView.html">Script2DView</a><span class="api-punctuation">,</span> <span class="api-parameter">context</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">)</span></code> {#property-on_draw}

<div class="api-description">

Draws using evaluated properties and read-only state.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">on_bounds</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Script2DView.html">Script2DView</a><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="RectValue.html">RectValue</a></code> {#property-on_bounds}

<div class="api-description">

Returns local bounds.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">on_update</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Script2DView.html">Script2DView</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">,</span> <span class="api-parameter">inputs</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">)</span></code> {#property-on_update}

<div class="api-description">

Updates self.state at fixed steps; own values must be declared in inputs when inputs are explicit. Mutable external upvalues are rejected.

</div>

</section>

</div>

</div>

:::
