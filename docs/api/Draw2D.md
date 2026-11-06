---
editLink: false
outline: 2
---

# Draw2D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Local 2D drawing context available only during scripted draw callbacks.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">save</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#draw2d-save}

<div class="api-description">

Saves the drawing transform and clip for a later :restore().

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">restore</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#draw2d-restore}

<div class="api-description">

Restores the most recently saved drawing state or closes a saved opacity layer.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">layer</span><span class="api-punctuation">(</span><span class="api-parameter">opacity</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#draw2d-layer-opacity}

<div class="api-description">

Begins a composited layer with the supplied opacity.
Call :restore() to composite all drawings in that layer once.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">translate</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#draw2d-translate-x-y}

<div class="api-description">

Translates subsequent drawing operations in local coordinates.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">scale</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#draw2d-scale-x-y}

<div class="api-description">

Scales subsequent drawing operations on each axis.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">rotate</span><span class="api-punctuation">(</span><span class="api-parameter">radians</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#draw2d-rotate-radians}

<div class="api-description">

Rotates subsequent drawing operations by an angle in radians.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">clip</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">)</span></code> {#draw2d-clip-path}

<div class="api-description">

Intersects the current clip with a drawing path.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">fill</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">, </span><span class="api-parameter">paint</span><span class="api-punctuation">: </span><span class="api-type-name">Paint</span><span class="api-punctuation">)</span></code> {#draw2d-fill-path-paint}

<div class="api-description">

Fills a path with a color or linear gradient paint.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">stroke</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><a class="api-type-name" href="Path.html">Path</a><span class="api-punctuation">, </span><span class="api-parameter">paint</span><span class="api-punctuation">: </span><span class="api-type-name">Paint</span><span class="api-punctuation">, </span><span class="api-parameter">width</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#draw2d-stroke-path-paint-width}

<div class="api-description">

Outlines a path with a color or linear gradient and a nonnegative width.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">rect</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">width</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">height</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">paint</span><span class="api-punctuation">: </span><span class="api-type-name">Paint</span><span class="api-punctuation">)</span></code> {#draw2d-rect-x-y-width-height-paint}

<div class="api-description">

Fills a rectangle in local coordinates with a color or gradient.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">circle</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">radius</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">paint</span><span class="api-punctuation">: </span><span class="api-type-name">Paint</span><span class="api-punctuation">)</span></code> {#draw2d-circle-x-y-radius-paint}

<div class="api-description">

Fills a circle in local coordinates with a color or gradient.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">image</span><span class="api-punctuation">(</span><span class="api-parameter">image</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image.html">Image</a><span class="api-punctuation">, </span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">width</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">height</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#draw2d-image-image-x-y-width-height}

<div class="api-description">

Draws an image into the supplied local rectangle.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw2D.html">Draw2D</a><span class="api-punctuation">:</span><span class="api-function">text</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">, </span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">size</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">paint</span><span class="api-punctuation">: </span><span class="api-type-name">Paint</span><span class="api-punctuation">, </span><span class="api-parameter">font</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Font.html">Font</a><span class="api-punctuation">)</span></code> {#draw2d-text-text-x-y-size-paint-font}

<div class="api-description">

Draws text at local coordinates with a size, paint, and optional font.
When omitted, a default system drawing font is used.

</div>

</section>

</div>

:::
