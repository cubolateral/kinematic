---
editLink: false
outline: 2
---

# Spatial3D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Shared spatial queries for compatible objects.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a></code>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">box_size</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-box_size}

<div class="api-description">

Returns the local bounding box dimensions as a vector.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">global_position</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-global_position}

<div class="api-description">

Returns the position composed through the object hierarchy.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">global_scale</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-global_scale}

<div class="api-description">

Returns the scale composed through the object hierarchy.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">global_rotation</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span> <span class="api-punctuation">|</span> <span class="api-type-name">number</span></code> {#spatial3d-global_rotation}

<div class="api-description">

Returns the rotation composed through the object hierarchy.
The result is radians for a 2D object or a quaternion for a 3D object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">middle</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-middle}

<div class="api-description">

Returns the middle anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">middle_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-middle_to-position}

<div class="api-description">

Returns the local position that places the middle anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-left}

<div class="api-description">

Returns the left anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">left_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-left_to-position}

<div class="api-description">

Returns the local position that places the left anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-right}

<div class="api-description">

Returns the right anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">right_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-right_to-position}

<div class="api-description">

Returns the local position that places the right anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top}

<div class="api-description">

Returns the top anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_to-position}

<div class="api-description">

Returns the local position that places the top anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom}

<div class="api-description">

Returns the bottom anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_to-position}

<div class="api-description">

Returns the local position that places the bottom anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_left}

<div class="api-description">

Returns the top_left anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_left_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_left_to-position}

<div class="api-description">

Returns the local position that places the top_left anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_right}

<div class="api-description">

Returns the top_right anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_right_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_right_to-position}

<div class="api-description">

Returns the local position that places the top_right anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_left}

<div class="api-description">

Returns the bottom_left anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_left_to-position}

<div class="api-description">

Returns the local position that places the bottom_left anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_right}

<div class="api-description">

Returns the bottom_right anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_right_to-position}

<div class="api-description">

Returns the local position that places the bottom_right anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-front}

<div class="api-description">

Returns the front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-front_to-position}

<div class="api-description">

Returns the local position that places the front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-back}

<div class="api-description">

Returns the back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-back_to-position}

<div class="api-description">

Returns the local position that places the back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">left_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-left_front}

<div class="api-description">

Returns the left_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">left_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-left_front_to-position}

<div class="api-description">

Returns the local position that places the left_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">right_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-right_front}

<div class="api-description">

Returns the right_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">right_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-right_front_to-position}

<div class="api-description">

Returns the local position that places the right_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_front}

<div class="api-description">

Returns the top_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_front_to-position}

<div class="api-description">

Returns the local position that places the top_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_front}

<div class="api-description">

Returns the bottom_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_front_to-position}

<div class="api-description">

Returns the local position that places the bottom_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_left_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_left_front}

<div class="api-description">

Returns the top_left_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_left_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_left_front_to-position}

<div class="api-description">

Returns the local position that places the top_left_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_right_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_right_front}

<div class="api-description">

Returns the top_right_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_right_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_right_front_to-position}

<div class="api-description">

Returns the local position that places the top_right_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_left_front}

<div class="api-description">

Returns the bottom_left_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_left_front_to-position}

<div class="api-description">

Returns the local position that places the bottom_left_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right_front</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_right_front}

<div class="api-description">

Returns the bottom_right_front anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right_front_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_right_front_to-position}

<div class="api-description">

Returns the local position that places the bottom_right_front anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">left_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-left_back}

<div class="api-description">

Returns the left_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">left_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-left_back_to-position}

<div class="api-description">

Returns the local position that places the left_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">right_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-right_back}

<div class="api-description">

Returns the right_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">right_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-right_back_to-position}

<div class="api-description">

Returns the local position that places the right_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_back}

<div class="api-description">

Returns the top_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_back_to-position}

<div class="api-description">

Returns the local position that places the top_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_back}

<div class="api-description">

Returns the bottom_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_back_to-position}

<div class="api-description">

Returns the local position that places the bottom_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_left_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_left_back}

<div class="api-description">

Returns the top_left_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_left_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_left_back_to-position}

<div class="api-description">

Returns the local position that places the top_left_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_right_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_right_back}

<div class="api-description">

Returns the top_right_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">top_right_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-top_right_back_to-position}

<div class="api-description">

Returns the local position that places the top_right_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_left_back}

<div class="api-description">

Returns the bottom_left_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_left_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_left_back_to-position}

<div class="api-description">

Returns the local position that places the bottom_left_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right_back</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_right_back}

<div class="api-description">

Returns the bottom_right_back anchor of the bounding box in parent coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Spatial3D.html">Spatial3D</a><span class="api-punctuation">:</span><span class="api-function">bottom_right_back_to</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">Value</span></code> {#spatial3d-bottom_right_back_to-position}

<div class="api-description">

Returns the local position that places the bottom_right_back anchor at position.

<div class="api-arguments">

- <span class="api-parameter">position</span>: Target position in parent coordinates.

</div>

</div>

</section>

</div>

:::
