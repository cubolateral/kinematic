---
editLink: false
outline: 2
---

# Script3D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Scripted 3D object with mesh drawing, tracked properties, and fixed-step simulation.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">signal</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Script3D.html">Script3D</a><span class="api-punctuation">,</span> <span class="api-parameter">callback</span><span class="api-punctuation">:</span> <span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">object</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Script3D.html">Script3D</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Signal.html">Signal</a></code> {#property-signal}

<div class="api-description">

Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.

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

### <code class="api-signature"><span class="api-name">auto_update</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-auto_update}

<div class="api-description">

Whether [`SimulationState::on_update`] runs automatically on project frames. Drawing is unaffected, so a paused simulation keeps showing its latest state. During seek replay, the value is sampled from its historical track at every frame.

</div>

</section>

</div>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script3D.html">Script3D</a><span class="api-punctuation">:</span><span class="api-function">inputs</span><span class="api-punctuation">(</span><span class="api-parameter">inputs</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <a class="api-type-name" href="Property.html">Property</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Object.html">Object</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Audio.html">Audio</a> <span class="api-punctuation">|</span> <span class="api-type-name">string</span><span class="api-punctuation">&gt;</span><span class="api-punctuation">)</span></code> {#script3d-inputs-inputs}

<div class="api-description">

Sets simulation dependencies from own property names, property handles, or other simulation objects.
Dependencies are sampled at each fixed step. Cyclic dependencies are rejected.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script3D.html">Script3D</a><span class="api-punctuation">:</span><span class="api-function">state</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">table</span></code> {#script3d-state}

<div class="api-description">

Returns an independent copy of the simulation state for inspection.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script3D.html">Script3D</a><span class="api-punctuation">:</span><span class="api-function">update</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#script3d-update}

<div class="api-description">

Schedules one explicit simulation update at the current time.
Use auto_update to control automatic fixed-step updates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script3D.html">Script3D</a><span class="api-punctuation">:</span><span class="api-function">checkpoint</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#script3d-checkpoint}

<div class="api-description">

Schedules a simulation checkpoint at the current time for later seek replay.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script3D.html">Script3D</a><span class="api-punctuation">:</span><span class="api-function">mutate</span><span class="api-punctuation">(</span><span class="api-parameter">callback</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">state</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#script3d-mutate-callback}

<div class="api-description">

Schedules a callback that changes simulation state at the current time.
Mutable data must live in state; callbacks cannot capture mutable external upvalues.

</div>

</section>

</div>

:::
