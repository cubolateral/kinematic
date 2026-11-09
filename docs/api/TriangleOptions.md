---
editLink: false
outline: 2
---

# TriangleOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Initial properties and resources for triangle.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="ResourceOptions.html">ResourceOptions</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">id</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#property-id}

<div class="api-description">

Stable identifier used to preserve object identity across reloads.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">name</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#property-name}

<div class="api-description">

Display name in the scene tree.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">size</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">256</span><span class="api-punctuation">,</span> <span class="api-literal">256</span><span class="api-punctuation">}</span></code> {#property-size}

<div class="api-description">

Base width and height in local coordinates.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">apex</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0.5</span></code> {#property-apex}

<div class="api-description">

Horizontal apex position from left (0) to right (1). Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">fill</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-fill}

<div class="api-description">

Color used to fill the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-stroke}

<div class="api-description">

Color used to outline the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke_width</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-stroke_width}

<div class="api-description">

Width used to outline the style. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">position</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-position}

<div class="api-description">

Position of the entity.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">origin</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-origin}

<div class="api-description">

Pivot relative to half the entity size. Values are not clamped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">scale</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-scale}

<div class="api-description">

Scale of the entity on each axis.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">rotation</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-rotation}

<div class="api-description">

Rotation of the entity in radians.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">opacity</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-opacity}

<div class="api-description">

Transparency applied while drawing this entity, from `0` to `1`. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">z_index</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-z_index}

<div class="api-description">

Stacking order among sibling objects. Higher values are drawn in front.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">follows_camera</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-follows_camera}

<div class="api-description">

Whether this entity and its subtree are affected by the canvas camera.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">blur</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-blur}

<div class="api-description">

Gaussian blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_offset</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_offset}

<div class="api-description">

Shadow offset in local canvas units.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_blur</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-shadow_blur}

<div class="api-description">

Shadow blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_color}

<div class="api-description">

Shadow color.

</div>

</section>

</div>

</div>

:::
