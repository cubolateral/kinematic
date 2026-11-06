---
editLink: false
outline: 2
---

# ResourceOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Optional resource configuration shared by native object constructors.

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">source</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image.html">Image</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Texture.html">Texture</a></code> {#property-source}

<div class="api-description">

Image resource for image objects, or live canvas texture for projection objects.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">font</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Font.html">Font</a></code> {#property-font}

<div class="api-description">

Project font for Text2D, Text3D, or Code; LaTeX uses its mathematical fonts.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shader</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Shader.html">Shader</a></code> {#property-shader}

<div class="api-description">

Image or mesh shader resource.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">uniforms</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <span class="api-type-name">boolean</span> <span class="api-punctuation">|</span> <span class="api-type-name">number</span> <span class="api-punctuation">|</span> <span class="api-type-name">Value</span><span class="api-punctuation">&gt;</span></code> {#property-uniforms}

<div class="api-description">

Initial typed shader uniform values; requires shader.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">textures</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <a class="api-type-name" href="Texture.html">Texture</a><span class="api-punctuation">&gt;</span></code> {#property-textures}

<div class="api-description">

Named live textures for image shaders; bindings require shader.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">resolution</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#property-resolution}

<div class="api-description">

Required width and height for additional canvases, each 1–8192 pixels.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">pixels_per_unit</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-pixels_per_unit}

<div class="api-description">

Image/projection pixels per local 3D unit; positive and finite, default 256.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">segments</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span></code> {#property-segments}

<div class="api-description">

Fixed sphere subdivision count, 3–256; default 32.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shader_padding</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-shader_padding}

<div class="api-description">

Extra image capture or mesh bounds padding; finite and nonnegative, default 0.

</div>

</section>

</div>

</div>

:::
