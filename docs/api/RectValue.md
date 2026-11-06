---
editLink: false
outline: 2
---

# RectValue

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Immutable rectangle value with coordinate, dimension, and anchor queries.

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-x}

<div class="api-description">

X component or left coordinate.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-y}

<div class="api-description">

Y component or bottom coordinate.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">width</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-width}

<div class="api-description">

Rectangle width in local coordinates.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">height</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-height}

<div class="api-description">

Rectangle height in local coordinates.

</div>

</section>

</div>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">middle</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-middle}

<div class="api-description">

Returns the middle point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">top</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-top}

<div class="api-description">

Returns the top point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">bottom</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-bottom}

<div class="api-description">

Returns the bottom point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-left}

<div class="api-description">

Returns the left point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-right}

<div class="api-description">

Returns the right point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">top_left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-top_left}

<div class="api-description">

Returns the top_left point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">bottom_right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-bottom_right}

<div class="api-description">

Returns the bottom_right point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">top_right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-top_right}

<div class="api-description">

Returns the top_right point of this rectangle as a vec2.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="RectValue.html">RectValue</a><span class="api-punctuation">:</span><span class="api-function">bottom_left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#rectvalue-bottom_left}

<div class="api-description">

Returns the bottom_left point of this rectangle as a vec2.

</div>

</section>

</div>

:::
