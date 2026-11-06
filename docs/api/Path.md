---
editLink: false
outline: 2
---

# Path

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Mutable path used by local 2D drawing operations.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">:</span><span class="api-function">move_to</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#path-move_to-x-y}

<div class="api-description">

Starts a contour at the supplied local coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">:</span><span class="api-function">line_to</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#path-line_to-x-y}

<div class="api-description">

Adds a straight segment from the current point to x, y.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">:</span><span class="api-function">quad_to</span><span class="api-punctuation">(</span><span class="api-parameter">x1</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y1</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">x2</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y2</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#path-quad_to-x1-y1-x2-y2}

<div class="api-description">

Adds a quadratic Bezier segment using x1, y1 as control point and x2, y2 as endpoint.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">:</span><span class="api-function">cubic_to</span><span class="api-punctuation">(</span><span class="api-parameter">x1</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y1</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">x2</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y2</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">x3</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y3</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#path-cubic_to-x1-y1-x2-y2-x3-y3}

<div class="api-description">

Adds a cubic Bezier segment with two control points and x3, y3 as endpoint.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">:</span><span class="api-function">close</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#path-close}

<div class="api-description">

Closes the current contour by connecting it to its starting point.

</div>

</section>

</div>

:::
