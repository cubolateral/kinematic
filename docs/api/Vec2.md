---
editLink: false
outline: 2
---

# Vec2

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Immutable Vec2 value.

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-x}

<div class="api-description">

The x component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-y}

<div class="api-description">

The y component.

</div>

</section>

</div>

**Operators**

<code class="api-signature"><span class="api-type-name">add</span><span class="api-punctuation">(</span><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2.html">Vec2</a></code>

<code class="api-signature"><span class="api-type-name">sub</span><span class="api-punctuation">(</span><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2.html">Vec2</a></code>

<code class="api-signature"><span class="api-type-name">mul</span><span class="api-punctuation">(</span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2.html">Vec2</a></code>

<code class="api-signature"><span class="api-type-name">div</span><span class="api-punctuation">(</span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2.html">Vec2</a></code>

<code class="api-signature"><span class="api-parameter">unm</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2.html">Vec2</a></code>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">:</span><span class="api-function">lerp</span><span class="api-punctuation">(</span><span class="api-parameter">other</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">, </span><span class="api-parameter">t</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#vec2-lerp-other-t}

<div class="api-description">

Interpolates between compatible values.

<div class="api-arguments">

- <span class="api-parameter">t</span>: Interpolation factor.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">:</span><span class="api-function">dot</span><span class="api-punctuation">(</span><span class="api-parameter">other</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#vec2-dot-other}

<div class="api-description">

Returns the dot product.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">:</span><span class="api-function">length</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#vec2-length}

<div class="api-description">

Returns the vector magnitude.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">:</span><span class="api-function">normalized</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#vec2-normalized}

<div class="api-description">

Returns a normalized value. Zero vectors remain zero.

</div>

</section>

</div>

:::
