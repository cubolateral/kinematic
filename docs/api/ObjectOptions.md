---
editLink: false
outline: 2
---

# ObjectOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Shared scripted object declaration metadata.

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">components</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Component.html">Component</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#property-components}

<div class="api-description">

Named custom tracked field declarations.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">state</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">table</span></code> {#property-state}

<div class="api-description">

Initial state copied independently for each instance.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">inputs</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <a class="api-type-name" href="Property.html">Property</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Object.html">Object</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Audio.html">Audio</a> <span class="api-punctuation">|</span> <span class="api-type-name">string</span><span class="api-punctuation">&gt;</span></code> {#property-inputs}

<div class="api-description">

Declared simulation dependencies; omitted inputs use own properties.

</div>

</section>

</div>

</div>

:::
