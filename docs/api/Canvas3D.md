---
editLink: false
outline: 2
---

# Canvas3D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Independent 3D perspective viewport with its own camera.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">signal</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Canvas3D.html">Canvas3D</a><span class="api-punctuation">,</span> <span class="api-parameter">callback</span><span class="api-punctuation">:</span> <span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">object</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Canvas3D.html">Canvas3D</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Signal.html">Signal</a></code> {#property-signal}

<div class="api-description">

Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">clear</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-clear}

<div class="api-description">

Color used to clear the canvas before drawing its children.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">filter</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-string">&#x27;nearest&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;linear&#x27;</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;nearest&quot;</span></code> {#property-filter}

<div class="api-description">

Sampling used by previews and projections of this canvas. Defaults to "nearest".

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_mode</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-string">&#x27;perspective&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;orthogonal&#x27;</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;perspective&quot;</span></code> {#property-camera_mode}

<div class="api-description">

Projection mode.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_position</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3Property.html">Vec3Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">3</span><span class="api-punctuation">}</span></code> {#property-camera_position}

<div class="api-description">

Position in canvas coordinates. Defaults to `(0, 0, 3)`.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_rotation</span><span class="api-punctuation">: </span><a class="api-type-name" href="QuaternionProperty.html">QuaternionProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-camera_rotation}

<div class="api-description">

Orientation looking along local negative Z, with positive Y up.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_fov</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0.7853982</span></code> {#property-camera_fov}

<div class="api-description">

Vertical field of view in radians, or view height in orthogonal mode. Minimum: 0.000001.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_near</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0.1</span></code> {#property-camera_near}

<div class="api-description">

Near clipping plane distance. Minimum: 0.000001.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">camera_far</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">1000</span></code> {#property-camera_far}

<div class="api-description">

Far clipping plane distance. Minimum: 0.000001.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">ambient_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-ambient_color}

<div class="api-description">

Ambient light color. Alpha does not affect lighting.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">ambient_intensity</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0.4</span></code> {#property-ambient_intensity}

<div class="api-description">

Nonnegative ambient light intensity multiplier. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

</div>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Canvas3D.html">Canvas3D</a><span class="api-punctuation">:</span><span class="api-function">focus_camera</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><span class="api-punctuation">{</span><span class="api-parameter">position</span><span class="api-punctuation">:</span> <span class="api-type-name">Value</span><span class="api-punctuation">,</span> <span class="api-parameter">target</span><span class="api-punctuation">:</span> <span class="api-type-name">Value</span><span class="api-punctuation">}</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#canvas3d-focus_camera-options}

<div class="api-description">

Creates an animation moving a 3D canvas camera to position and aiming at target.
Both values must be vec3. Call :play to schedule it.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Canvas3D.html">Canvas3D</a><span class="api-punctuation">:</span><span class="api-function">texture</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Texture.html">Texture</a></code> {#canvas3d-texture}

<div class="api-description">

Returns the live texture produced by a canvas object.
Use it as another object source or shader texture input.

</div>

</section>

</div>

:::
