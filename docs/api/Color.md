---
editLink: false
outline: 2
---

# Color

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Immutable Color value.

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">r</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-r}

<div class="api-description">

The r component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">g</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-g}

<div class="api-description">

The g component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">b</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-b}

<div class="api-description">

The b component.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">a</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-a}

<div class="api-description">

The a component.

</div>

</section>

</div>

**Operators**

<code class="api-signature"><span class="api-type-name">add</span><span class="api-punctuation">(</span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Color.html">Color</a></code>

<code class="api-signature"><span class="api-type-name">sub</span><span class="api-punctuation">(</span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Color.html">Color</a></code>

<code class="api-signature"><span class="api-type-name">mul</span><span class="api-punctuation">(</span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Color.html">Color</a></code>

<code class="api-signature"><span class="api-type-name">div</span><span class="api-punctuation">(</span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Color.html">Color</a></code>

<code class="api-signature"><span class="api-parameter">unm</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Color.html">Color</a></code>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">:</span><span class="api-function">lerp</span><span class="api-punctuation">(</span><span class="api-parameter">other</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">, </span><span class="api-parameter">t</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a></code> {#color-lerp-other-t}

<div class="api-description">

Interpolates between compatible values.

<div class="api-arguments">

- <span class="api-parameter">t</span>: Interpolation factor.

</div>

</div>

</section>

</div>

:::
