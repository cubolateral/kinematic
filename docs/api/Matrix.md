---
editLink: false
outline: 2
---

# Matrix

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Immutable column-major 4x4 transformation matrix.

**Operators**

<code class="api-signature"><span class="api-type-name">mul</span><span class="api-punctuation">(</span><a class="api-type-name" href="Matrix.html">Matrix</a><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Matrix.html">Matrix</a></code>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Matrix.html">Matrix</a><span class="api-punctuation">:</span><span class="api-function">translate</span><span class="api-punctuation">(</span><span class="api-parameter">value</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Matrix.html">Matrix</a></code> {#matrix-translate-value}

<div class="api-description">

Returns this matrix multiplied by a translation matrix.
The translation must be a vec3; the original matrix is unchanged.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Matrix.html">Matrix</a><span class="api-punctuation">:</span><span class="api-function">scale</span><span class="api-punctuation">(</span><span class="api-parameter">value</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Matrix.html">Matrix</a></code> {#matrix-scale-value}

<div class="api-description">

Returns this matrix multiplied by a scale matrix.
The scale must be a vec3; the original matrix is unchanged.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Matrix.html">Matrix</a><span class="api-punctuation">:</span><span class="api-function">rotate</span><span class="api-punctuation">(</span><span class="api-parameter">quaternion</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Matrix.html">Matrix</a></code> {#matrix-rotate-quaternion}

<div class="api-description">

Returns this matrix multiplied by a quaternion rotation matrix.
The original matrix is unchanged.

</div>

</section>

</div>

:::
