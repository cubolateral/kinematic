---
editLink: false
outline: 2
---

# Container3DOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Initial properties and resources for container_3d.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="ResourceOptions.html">ResourceOptions</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">id</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#property-id}

<div class="api-description">

Stable identifier used to preserve object identity across reloads.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">name</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#property-name}

<div class="api-description">

Display name in the scene tree.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">gap</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">z</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-gap}

<div class="api-description">

Spacing between children on each layout axis in local coordinates.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">direction</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-string">&#x27;horizontal&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;horizontal_reverse&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;vertical&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;vertical_reverse&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;depth&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;depth_reverse&#x27;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;horizontal&quot;</span></code> {#property-direction}

<div class="api-description">

Axis and direction used to arrange the container children.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">position</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">z</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-position}

<div class="api-description">

Position relative to the parent in local coordinates.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">origin</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">z</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-origin}

<div class="api-description">

Pivot relative to half the entity size. Values are not clamped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">rotation</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Quaternion.html">Quaternion</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-rotation}

<div class="api-description">

Local orientation as a quaternion. Use k.axis_angle to construct a rotation.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">scale</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">z</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-scale}

<div class="api-description">

Local scale multiplier on each axis.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

</div>

</div>

:::
