---
editLink: false
outline: 2
---

# Projection3D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Unlit 3D plane sampling the premultiplied output of a canvas.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">signal</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Projection3D.html">Projection3D</a><span class="api-punctuation">,</span> <span class="api-parameter">callback</span><span class="api-punctuation">:</span> <span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">object</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Projection3D.html">Projection3D</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Signal.html">Signal</a></code> {#property-signal}

<div class="api-description">

Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">size</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-size}

<div class="api-description">

Geometry dimensions in local coordinates.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">position</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-position}

<div class="api-description">

Position relative to the parent in local coordinates.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">origin</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-origin}

<div class="api-description">

Pivot relative to half the entity size. Values are not clamped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">rotation</span><span class="api-punctuation">: </span><a class="api-type-name" href="QuaternionProperty.html">QuaternionProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-rotation}

<div class="api-description">

Local orientation as a quaternion. Use k.axis_angle to construct a rotation.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">scale</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-scale}

<div class="api-description">

Local scale multiplier on each axis.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

</div>

</div>

:::
