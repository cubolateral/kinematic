---
editLink: false
outline: 2
---

# k

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">audio_synth</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioSynthOptions.html">AudioSynthOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioResource.html">AudioResource</a></code> {#k-audio_synth-options}

<div class="api-description">

Creates a silent synthesizer resource; the envelope must last 0.001–10 seconds.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">audio_file</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioResource.html">AudioResource</a></code> {#k-audio_file-path}

<div class="api-description">

Loads a project-relative WAV, MP3, OGG, or FLAC file using FFmpeg; creation does not play it.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">vec2</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a></code> {#k-vec2-x-y}

<div class="api-description">

Creates a two-dimensional value with x and y components.
Use it for positions, dimensions, and other vector properties.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">vec3</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">z</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a></code> {#k-vec3-x-y-z}

<div class="api-description">

Creates a three-dimensional value with x, y, and z components.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">color</span><span class="api-punctuation">(</span><span class="api-parameter">color</span><span class="api-punctuation">: </span><span class="api-type-name">string</span> <span class="api-punctuation">|</span> <span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a></code> {#k-color-color-g-b-a}

<div class="api-description">

Creates a color from RGB or RGBA numbers, or a six/eight-digit hexadecimal string.
Numeric components use the 0 to 1 convention; omitted alpha defaults to 1. Example: k.color("#60A5FA") or k.color(1,0,0,0.5).

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">quat</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">z</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">w</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Quaternion.html">Quaternion</a></code> {#k-quat-x-y-z-w}

<div class="api-description">

Creates a quaternion from x, y, z, and w components.
Use k.axis_angle for an axis and an angle in radians.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">component</span><span class="api-punctuation">(</span><span class="api-parameter">fields</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <a class="api-type-name" href="PropertyDefinition.html">PropertyDefinition</a><span class="api-punctuation">&gt;</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Component.html">Component</a></code> {#k-component-fields}

<div class="api-description">

Custom property schema; the required default value determines its type.
Declares custom animatable fields from default values and optional numeric bounds or string choices.
The default determines each field type. Custom names must not shadow native properties.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">object</span><span class="api-punctuation">(</span><span class="api-parameter">definition</span><span class="api-punctuation">: </span><a class="api-type-name" href="ObjectOptions.html">ObjectOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Script2D.html">Script2D</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="Script3D.html">Script3D</a></code> {#k-object-definition}

<div class="api-description">

Scripted object callbacks, state, components, and methods.
Declares a reusable scripted 2D or 3D object with components, state, callbacks, and methods.
Instantiate it with scene:object(definition, options). Drawing reads state; updates change state at fixed steps.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">axis_angle</span><span class="api-punctuation">(</span><span class="api-parameter">axis</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">, </span><span class="api-parameter">radians</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Quaternion.html">Quaternion</a></code> {#k-axis_angle-axis-radians}

<div class="api-description">

Creates a quaternion from a nonzero vec3 axis and a finite angle in radians.
The axis is normalized automatically.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">quad</span><span class="api-punctuation">(</span><span class="api-parameter">a</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">b</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">c</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">d</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Quad.html">Quad</a></code> {#k-quad-a-b-c-d}

<div class="api-description">

Creates a four-component value with a, b, c, and d channels.
For rectangle rounding, channels correspond to top-left, top-right, bottom-right, and bottom-left corners.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">rect</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">width</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">height</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RectValue.html">RectValue</a></code> {#k-rect-x-y-width-height}

<div class="api-description">

Creates an immutable rectangle value from local coordinates and nonnegative dimensions.
This is a value helper; use scene:rect(options) to create a scene object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">random</span><span class="api-punctuation">(</span><span class="api-parameter">seed</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Random.html">Random</a></code> {#k-random-seed}

<div class="api-description">

Creates a deterministic random generator from a nonnegative integer seed.
Store it in simulation state to preserve its sequence during seek replay.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">path</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Path.html">Path</a></code> {#k-path}

<div class="api-description">

Creates a mutable drawing path.
Use move_to, line_to, quad_to, cubic_to, and close to build its geometry.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">mat4</span><span class="api-punctuation">(</span><span class="api-parameter">columns</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Matrix.html">Matrix</a></code> {#k-mat4-columns}

<div class="api-description">

Creates a matrix from sixteen column-major numbers, or the identity when omitted.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">mesh</span><span class="api-punctuation">(</span><span class="api-parameter">data</span><span class="api-punctuation">: </span><span class="api-punctuation">{</span><span class="api-parameter">vertices</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">,</span> <span class="api-parameter">indices</span><span class="api-punctuation">:</span> <span class="api-type-name">integer</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">,</span> <span class="api-parameter">normals</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">,</span> <span class="api-parameter">uvs</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">,</span> <span class="api-parameter">colors</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">Value</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">}</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Mesh.html">Mesh</a></code> {#k-mesh-data}

<div class="api-description">

Immutable indexed triangle geometry used by 3D drawing.
Creates an immutable indexed triangle mesh with one-based indices.
Vertices are vec3 number arrays; optional normals, UVs, and colors must match the vertex count. Normals are generated when omitted.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">image</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image.html">Image</a></code> {#k-image-path}

<div class="api-description">

Live texture produced by a canvas or projection.
Loads an image resource from a path relative to the project directory.
Use the returned handle as an image object source or in a drawing callback.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">font</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Font.html">Font</a></code> {#k-font-path}

<div class="api-description">

Loads a font resource from a path relative to the project directory.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">image_shader</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><span class="api-punctuation">{</span><span class="api-parameter">source</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">,</span> <span class="api-parameter">path</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">}</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Shader.html">Shader</a></code> {#k-image_shader-options}

<div class="api-description">

Creates a 2D image shader from inline source or a project-relative path.
When both are supplied, the file path takes precedence.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">mesh_shader</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><span class="api-punctuation">{</span><span class="api-parameter">fragment</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">,</span> <span class="api-parameter">fragment_path</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">,</span> <span class="api-parameter">vertex</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">,</span> <span class="api-parameter">vertex_path</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">}</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Shader.html">Shader</a></code> {#k-mesh_shader-options}

<div class="api-description">

Creates a 3D mesh shader with a required fragment source and an optional vertex source.
Sources may be inline or project-relative files; file paths take precedence.

</div>

</section>

</div>

:::
