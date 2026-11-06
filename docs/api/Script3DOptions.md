---
editLink: false
outline: 2
---

# Script3DOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Initial properties and resources for script_3d.

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

<section class="api-member">

### <code class="api-signature"><span class="api-name">unlit</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">false</span></code> {#property-unlit}

<div class="api-description">

Draws the surface without lighting when true.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">albedo</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-albedo}

<div class="api-description">

Base surface color used by lighting.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">metallic</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-metallic}

<div class="api-description">

Metallic fraction, from zero for a dielectric to one for a metal. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">roughness</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0.7</span></code> {#property-roughness}

<div class="api-description">

Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Minimum: 0.04. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">outline_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-outline_color}

<div class="api-description">

Color of the mesh outline.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">outline_width</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-outline_width}

<div class="api-description">

Width of the mesh outline in screen pixels. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">opacity</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-opacity}

<div class="api-description">

Surface opacity, from zero for transparent to one for opaque. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">auto_update</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-auto_update}

<div class="api-description">

Whether [`SimulationState::on_update`] runs automatically on project frames. Drawing is unaffected, so a paused simulation keeps showing its latest state. During seek replay, the value is sampled from its historical track at every frame.

</div>

</section>

</div>

</div>

:::
