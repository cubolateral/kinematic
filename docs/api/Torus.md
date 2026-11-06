---
editLink: false
outline: 2
---

# Torus

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Torus centered on its local origin with its ring around the Y axis.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">signal</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Torus.html">Torus</a><span class="api-punctuation">,</span> <span class="api-parameter">callback</span><span class="api-punctuation">:</span> <span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">object</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Torus.html">Torus</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Signal.html">Signal</a></code> {#property-signal}

<div class="api-description">

Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">major_radius</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0.375</span></code> {#property-major_radius}

<div class="api-description">

Distance from the torus center to the center of its tube in local units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">minor_radius</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0.125</span></code> {#property-minor_radius}

<div class="api-description">

Radius of the torus tube in local units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">major_segments</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">integer</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">32</span></code> {#property-major_segments}

<div class="api-description">

Number of subdivisions around the torus ring, between 3 and 256. Minimum: 3. Maximum: 256.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">minor_segments</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">integer</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">16</span></code> {#property-minor_segments}

<div class="api-description">

Number of subdivisions around the torus tube, between 3 and 256. Minimum: 3. Maximum: 256.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">unlit</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">false</span></code> {#property-unlit}

<div class="api-description">

Draws the surface without lighting when true.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">albedo</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-albedo}

<div class="api-description">

Base surface color used by lighting.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">metallic</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-metallic}

<div class="api-description">

Metallic fraction, from zero for a dielectric to one for a metal. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">roughness</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0.7</span></code> {#property-roughness}

<div class="api-description">

Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Minimum: 0.04. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">outline_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-outline_color}

<div class="api-description">

Color of the mesh outline.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">outline_width</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-outline_width}

<div class="api-description">

Width of the mesh outline in screen pixels. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">opacity</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-opacity}

<div class="api-description">

Surface opacity, from zero for transparent to one for opaque. Minimum: 0. Maximum: 1.

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
