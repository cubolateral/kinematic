---
editLink: false
outline: 2
---

# PropertyDefinition

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">default</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span> <span class="api-punctuation">|</span> <span class="api-type-name">number</span> <span class="api-punctuation">|</span> <span class="api-type-name">string</span> <span class="api-punctuation">|</span> <span class="api-type-name">Value</span></code> {#property-default}

<div class="api-description">

Required initial value; determines the property type.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">min</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-min}

<div class="api-description">

Optional lower bound for a numeric property.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">max</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-max}

<div class="api-description">

Optional upper bound for a numeric property.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">choices</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#property-choices}

<div class="api-description">

Allowed snake_case values for a string property.

</div>

</section>

</div>

</div>

:::
