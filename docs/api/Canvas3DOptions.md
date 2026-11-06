---
editLink: false
outline: 2
---

# Canvas3DOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Initial properties and resources for canvas_3d.

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

### <code class="api-signature"><span class="api-name">camera_mode</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-string">&#x27;perspective&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;orthogonal&#x27;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;perspective&quot;</span></code> {#property-camera_mode}

<div class="api-description">

Projection mode.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_position</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">z</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">3</span><span class="api-punctuation">}</span></code> {#property-camera_position}

<div class="api-description">

Position in canvas coordinates. Defaults to `(0, 0, 3)`.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_rotation</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Quaternion.html">Quaternion</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-camera_rotation}

<div class="api-description">

Orientation looking along local negative Z, with positive Y up.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_fov</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0.7853982</span></code> {#property-camera_fov}

<div class="api-description">

Vertical field of view in radians, or view height in orthogonal mode. Minimum: 0.000001.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_near</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0.1</span></code> {#property-camera_near}

<div class="api-description">

Near clipping plane distance. Minimum: 0.000001.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_far</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">1000</span></code> {#property-camera_far}

<div class="api-description">

Far clipping plane distance. Minimum: 0.000001.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">ambient_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-ambient_color}

<div class="api-description">

Ambient light color. Alpha does not affect lighting.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">ambient_intensity</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0.4</span></code> {#property-ambient_intensity}

<div class="api-description">

Nonnegative ambient light intensity multiplier. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

</div>

</div>

:::
