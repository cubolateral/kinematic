---
editLink: false
outline: 2
---

# Draw3D

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Local 3D drawing context available only during scripted draw callbacks.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw3D.html">Draw3D</a><span class="api-punctuation">:</span><span class="api-function">mesh</span><span class="api-punctuation">(</span><span class="api-parameter">mesh</span><span class="api-punctuation">: </span><a class="api-type-name" href="Mesh.html">Mesh</a><span class="api-punctuation">, </span><span class="api-parameter">matrix</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Matrix.html">Matrix</a><span class="api-punctuation">)</span></code> {#draw3d-mesh-mesh-matrix}

<div class="api-description">

Draws a mesh using the optional local transformation matrix.
Drawing callbacks must not mutate animation or simulation state.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Draw3D.html">Draw3D</a><span class="api-punctuation">:</span><span class="api-function">instances</span><span class="api-punctuation">(</span><span class="api-parameter">mesh</span><span class="api-punctuation">: </span><a class="api-type-name" href="Mesh.html">Mesh</a><span class="api-punctuation">, </span><span class="api-parameter">matrices</span><span class="api-punctuation">: </span><a class="api-type-name" href="Matrix.html">Matrix</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">)</span></code> {#draw3d-instances-mesh-matrices}

<div class="api-description">

Draws the same mesh at each local transformation matrix.

</div>

</section>

</div>

:::
