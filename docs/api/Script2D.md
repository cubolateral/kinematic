---
editLink: false
outline: 2
---

# Script2D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Scripted 2D object with local drawing, tracked properties, and fixed-step simulation.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">signal</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Script2D.html">Script2D</a><span class="api-punctuation">,</span> <span class="api-parameter">callback</span><span class="api-punctuation">:</span> <span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">object</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Script2D.html">Script2D</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Signal.html">Signal</a></code> {#property-signal}

<div class="api-description">

Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">position</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-position}

<div class="api-description">

Position of the entity.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">origin</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-origin}

<div class="api-description">

Pivot relative to half the entity size. Values are not clamped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">scale</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-scale}

<div class="api-description">

Scale of the entity on each axis.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">rotation</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-rotation}

<div class="api-description">

Rotation of the entity in radians.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">opacity</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-opacity}

<div class="api-description">

Transparency applied while drawing this entity, from `0` to `1`. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">z_index</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">integer</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-z_index}

<div class="api-description">

Stacking order among sibling objects. Higher values are drawn in front.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">follows_camera</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-follows_camera}

<div class="api-description">

Whether this entity and its subtree are affected by the canvas camera.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">fill</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-fill}

<div class="api-description">

Color used to fill the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-stroke}

<div class="api-description">

Color used to outline the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke_width</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-stroke_width}

<div class="api-description">

Width used to outline the style. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">blur</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-blur}

<div class="api-description">

Gaussian blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_offset</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_offset}

<div class="api-description">

Shadow offset in local canvas units.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_blur</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-shadow_blur}

<div class="api-description">

Shadow blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_color}

<div class="api-description">

Shadow color.

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

## <code class="api-signature"><a class="api-type-name" href="Script2D.html">Script2D</a><span class="api-punctuation">:</span><span class="api-function">inputs</span><span class="api-punctuation">(</span><span class="api-parameter">inputs</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <a class="api-type-name" href="Property.html">Property</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Object.html">Object</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Audio.html">Audio</a> <span class="api-punctuation">|</span> <span class="api-type-name">string</span><span class="api-punctuation">&gt;</span><span class="api-punctuation">)</span></code> {#script2d-inputs-inputs}

<div class="api-description">

Sets simulation dependencies from own property names, property handles, or other simulation objects.
Dependencies are sampled at each fixed step. Cyclic dependencies are rejected.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script2D.html">Script2D</a><span class="api-punctuation">:</span><span class="api-function">state</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">table</span></code> {#script2d-state}

<div class="api-description">

Returns an independent copy of the simulation state for inspection.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script2D.html">Script2D</a><span class="api-punctuation">:</span><span class="api-function">update</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#script2d-update}

<div class="api-description">

Schedules one explicit simulation update at the current time.
Use auto_update to control automatic fixed-step updates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script2D.html">Script2D</a><span class="api-punctuation">:</span><span class="api-function">checkpoint</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#script2d-checkpoint}

<div class="api-description">

Schedules a simulation checkpoint at the current time for later seek replay.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Script2D.html">Script2D</a><span class="api-punctuation">:</span><span class="api-function">mutate</span><span class="api-punctuation">(</span><span class="api-parameter">callback</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">state</span><span class="api-punctuation">:</span> <span class="api-type-name">table</span><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#script2d-mutate-callback}

<div class="api-description">

Schedules a callback that changes simulation state at the current time.
Mutable data must live in state; callbacks cannot capture mutable external upvalues.

</div>

</section>

</div>

:::
