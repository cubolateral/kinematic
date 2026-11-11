---
editLink: false
outline: 2
---

# Spatial2D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Easing curve names accepted by Animation:play. The default is "cubic_in_out".
Shared spatial queries for compatible objects.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a></code>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">box_size</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-box_size}

<div class="api-description">

Returns the local bounding box dimensions as a vector.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">global_position</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-global_position}

<div class="api-description">

Returns the position composed through the object hierarchy.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">global_scale</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-global_scale}

<div class="api-description">

Returns the scale composed through the object hierarchy.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">global_rotation</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span> <span class="api-punctuation">|</span> <span class="api-type-name">number</span></code> {#spatial2d-global_rotation}

<div class="api-description">

Returns the rotation composed through the object hierarchy.
The result is radians for a 2D object or a quaternion for a 3D object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">global_opacity</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#spatial2d-global_opacity}

<div class="api-description">

Returns opacity composed through the hierarchy. Requires a 2D object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">on_bounds</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RectValue.html">RectValue</a></code> {#spatial2d-on_bounds}

<div class="api-description">

Returns the local bounds at the evaluated time; also used by the engine.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">middle</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-middle}

<div class="api-description">

Returns the middle anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">middle_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-middle_to-position}

<div class="api-description">

Returns the local position that places the middle anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-left}

<div class="api-description">

Returns the left anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">left_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-left_to-position}

<div class="api-description">

Returns the local position that places the left anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-right}

<div class="api-description">

Returns the right anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">right_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-right_to-position}

<div class="api-description">

Returns the local position that places the right anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">top</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-top}

<div class="api-description">

Returns the top anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">top_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-top_to-position}

<div class="api-description">

Returns the local position that places the top anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">bottom</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-bottom}

<div class="api-description">

Returns the bottom anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">bottom_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-bottom_to-position}

<div class="api-description">

Returns the local position that places the bottom anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">top_left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-top_left}

<div class="api-description">

Returns the top_left anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">top_left_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-top_left_to-position}

<div class="api-description">

Returns the local position that places the top_left anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">top_right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-top_right}

<div class="api-description">

Returns the top_right anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">top_right_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-top_right_to-position}

<div class="api-description">

Returns the local position that places the top_right anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-bottom_left}

<div class="api-description">

Returns the bottom_left anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-bottom_left_to-position}

<div class="api-description">

Returns the local position that places the bottom_left anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-bottom_right}

<div class="api-description">

Returns the bottom_right anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial2d-bottom_right_to-position}

<div class="api-description">

Returns the local position that places the bottom_right anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

</div>

:::
