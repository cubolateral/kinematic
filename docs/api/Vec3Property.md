---
editLink: false
outline: 2
---

# Vec3Property

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Animatable Vec3 property.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">&gt;</span></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">x</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span></code> {#property-x}

<div class="api-description">

Animatable x component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">y</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span></code> {#property-y}

<div class="api-description">

Animatable y component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">z</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span></code> {#property-z}

<div class="api-description">

Animatable z component.

</div>

</section>

</div>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation">:</span><span class="api-function">get</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a></code> {#vec3property-get}

<div class="api-description">

Reads the current value of this property or scalar channel.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation">:</span><span class="api-function">set</span><span class="api-punctuation">(</span><span class="api-parameter">value</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">)</span></code> {#vec3property-set-value}

<div class="api-description">

Sets the property or scalar channel immediately during construction.
Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation">:</span><span class="api-function">to</span><span class="api-punctuation">(</span><span class="api-parameter">value</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#vec3property-to-value}

<div class="api-description">

Creates an animation toward an absolute value.
Preparing the animation does not schedule it; call :play(duration, easing).

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation">:</span><span class="api-function">by</span><span class="api-punctuation">(</span><span class="api-parameter">delta</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#vec3property-by-delta}

<div class="api-description">

Creates an animation adding a relative increment.
Repeated :play calls apply the increment at each scheduled start.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation">:</span><span class="api-function">from</span><span class="api-punctuation">(</span><span class="api-parameter">pair</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#vec3property-from-pair}

<div class="api-description">

Creates an animation with explicit {start, finish} endpoints.
Both endpoints must match the property type. Call :play to schedule it.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation">:</span><span class="api-function">signal</span><span class="api-punctuation">(</span><span class="api-parameter">callback</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">property</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Signal.html">Signal</a></code> {#vec3property-signal-callback}

<div class="api-description">

Schedules a callback receiving this property and the frame after scene evaluation.
Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.

</div>

</section>

</div>

:::
