---
editLink: false
outline: 2
---

# ObjectOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">dimension</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-literal">2</span> <span class="api-punctuation">|</span> <span class="api-literal">3</span></code> {#property-dimension}

<div class="api-description">

Spatial dimension, 2 or 3; defaults to 2.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">components</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Component.html">Component</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#property-components}

<div class="api-description">

Custom tracked field declarations.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">state</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">table</span></code> {#property-state}

<div class="api-description">

Initial state copied independently for each instance.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">inputs</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <a class="api-type-name" href="Property.html">Property</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Object.html">Object</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Audio.html">Audio</a> <span class="api-punctuation">|</span> <span class="api-type-name">string</span><span class="api-punctuation">&gt;</span></code> {#property-inputs}

<div class="api-description">

Declared simulation dependencies. Omitted inputs use own properties; an empty table declares none.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">update</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">state</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">,</span> <span class="api-parameter">inputs</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">)</span></code> {#property-update}

<div class="api-description">

Fixed-step state update callback; external mutable upvalues are rejected.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">draw</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">context</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Draw2D.html">Draw2D</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Draw3D.html">Draw3D</a><span class="api-punctuation">,</span> <span class="api-parameter">properties</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">,</span> <span class="api-parameter">state</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">)</span></code> {#property-draw}

<div class="api-description">

Local drawing callback; reads evaluated properties and state.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">bounds</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">properties</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">,</span> <span class="api-parameter">state</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">z</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">width</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">height</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">depth</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span></code> {#property-bounds}

<div class="api-description">

Returns local coordinates and dimensions for selection and layout.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">name</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#property-name}

<div class="api-description">

Default display name of the scripted object.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">methods</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <span class="api-type-name">function</span><span class="api-punctuation">&gt;</span></code> {#property-methods}

<div class="api-description">

Instance methods invoked with the object as self; fundamental operations cannot be replaced.

</div>

</section>

</div>

</div>

:::
