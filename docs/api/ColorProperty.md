---
editLink: false
outline: 2
---

# ColorProperty

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Animatable Color property.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">&gt;</span></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">r</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span></code> {#property-r}

<div class="api-description">

Animatable r component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">g</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span></code> {#property-g}

<div class="api-description">

Animatable g component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">b</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span></code> {#property-b}

<div class="api-description">

Animatable b component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">a</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span></code> {#property-a}

<div class="api-description">

Animatable a component.

</div>

</section>

</div>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation">:</span><span class="api-function">get</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a></code> {#colorproperty-get}

<div class="api-description">

Reads the current value of this property or scalar channel.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation">:</span><span class="api-function">set</span><span class="api-punctuation">(</span><span class="api-parameter">value</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">)</span></code> {#colorproperty-set-value}

<div class="api-description">

Sets the property or scalar channel immediately during construction.
Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation">:</span><span class="api-function">to</span><span class="api-punctuation">(</span><span class="api-parameter">value</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#colorproperty-to-value}

<div class="api-description">

Creates an animation toward an absolute value.
Preparing the animation does not schedule it; call :play(duration, easing).

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation">:</span><span class="api-function">by</span><span class="api-punctuation">(</span><span class="api-parameter">delta</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#colorproperty-by-delta}

<div class="api-description">

Creates an animation adding a relative increment.
Repeated :play calls apply the increment at each scheduled start.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation">:</span><span class="api-function">from</span><span class="api-punctuation">(</span><span class="api-parameter">pair</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#colorproperty-from-pair}

<div class="api-description">

Creates an animation with explicit {start, finish} endpoints.
Both endpoints must match the property type. Call :play to schedule it.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation">:</span><span class="api-function">signal</span><span class="api-punctuation">(</span><span class="api-parameter">callback</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">property</span><span class="api-punctuation">:</span> <a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Signal.html">Signal</a></code> {#colorproperty-signal-callback}

<div class="api-description">

Schedules a callback receiving this property and the frame after scene evaluation.
Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.

</div>

</section>

</div>

:::
