---
editLink: false
outline: 2
---

# Canvas2DOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Initial properties and resources for canvas_2d.

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

### <code class="api-signature"><span class="api-name">clear</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-clear}

<div class="api-description">

Color used to clear the canvas before drawing its children.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">filter</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-string">&#x27;nearest&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;linear&#x27;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;nearest&quot;</span></code> {#property-filter}

<div class="api-description">

Sampling used by previews and projections of this canvas. Defaults to "nearest".

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_position</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-camera_position}

<div class="api-description">

Position observed at the center of the viewport.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_zoom</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-camera_zoom}

<div class="api-description">

Magnification applied to the scene. Minimum: 0.000001.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_rotation</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-camera_rotation}

<div class="api-description">

Rotation of the view in radians.

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

</div>

</div>

:::
