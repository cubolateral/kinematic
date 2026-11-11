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

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">audio</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioOptions.html">AudioOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Audio.html">Audio</a></code> {#k-audio-options}

<div class="api-description">

Creates a silent player. Scheduled tails contribute to scene duration; repeat scopes are rejected.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">view</span><span class="api-punctuation">(</span><span class="api-parameter">dimension</span><span class="api-punctuation">: </span><span class="api-string">&quot;2d&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;3d&quot;</span><span class="api-punctuation">)</span></code> {#k-view-dimension}

<div class="api-description">

Schedules an instantaneous switch between the built-in 2D and 3D world canvases.
The dimension must be "2d" or "3d".

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">wait</span><span class="api-punctuation">(</span><span class="api-parameter">seconds</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#k-wait-seconds}

<div class="api-description">

Advances the current scheduling scope by a nonnegative duration in seconds.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">event</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span></code> {#k-event-name}

<div class="api-description">

Adds a named editable wait at the current timeline position.
Its duration can be adjusted in the editor.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">all</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#k-all-schedule}

<div class="api-description">

Schedules the callback animations simultaneously.
The parent timeline advances by the longest child duration.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">chain</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#k-chain-schedule}

<div class="api-description">

Schedules the callback animations sequentially on the current timeline.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">parallel</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#k-parallel-schedule}

<div class="api-description">

Schedules the callback without advancing or extending the parent timeline.
Animations inside the callback are sequential unless grouped with k.all.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">loop</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#k-loop-schedule}

<div class="api-description">

Repeats one positive finite animation cycle without advancing the parent timeline.
The callback executes once; its animations repeat until their lifetime ends. Nested loops and object creation, attachment, or removal within a cycle are rejected.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">world_2d</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas2D.html">Canvas2D</a></code> {#k-world_2d}

<div class="api-description">

Returns the built-in 2D canvas.
Attach 2D objects with k.world_2d():add(object).

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">world_3d</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas3D.html">Canvas3D</a></code> {#k-world_3d}

<div class="api-description">

Returns the built-in 3D canvas.
Attach 3D objects with k.world_3d():add(object).

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">root</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a></code> {#k-root}

<div class="api-description">

Returns the internal root container shared by all scene contents.
Attach additional canvases here; use k.world_2d() and k.world_3d() for ordinary world objects.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">audio_synth</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioSynthOptions.html">AudioSynthOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioResource.html">AudioResource</a></code> {#k-audio_synth-options}

<div class="api-description">

Creates a silent synthesizer resource; the envelope must last 0.001–10 seconds.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">load_audio</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioResource.html">AudioResource</a></code> {#k-load_audio-path}

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

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">track</span><span class="api-punctuation">(</span><span class="api-parameter">initial</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">, </span><span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">T</span><span class="api-punctuation">&gt;</span></code> {#k-track-initial-id}

<div class="api-description">

Creates a standalone animatable property from an initial value.
An optional stable id preserves object identity across reloads. Supports get, set, to, by, from, and signal.

**Type parameter:** <code class="api-signature"><span class="api-type-name">T</span></code>

**Overloads**

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2Property.html">Vec2Property</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec3Property.html">Vec3Property</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="ColorProperty.html">ColorProperty</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Quaternion.html">Quaternion</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="QuaternionProperty.html">QuaternionProperty</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Quad.html">Quad</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="QuadProperty.html">QuadProperty</a></code>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">component</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">, </span><span class="api-parameter">fields</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">,</span> <a class="api-type-name" href="PropertyDefinition.html">PropertyDefinition</a><span class="api-punctuation">&gt;</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Component.html">Component</a></code> {#k-component-name-fields}

<div class="api-description">

Custom property schema; the required default value determines its type.
Declares a named component; its fields form a group in the Inspector.
Use capitalized words separated by spaces, such as "Motion 2D".

<div class="api-arguments">

- <span class="api-parameter">name</span>: Component display name, such as "Progress".

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">object_2d</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">, </span><span class="api-parameter">definition</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object2DOptions.html">Object2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object2DDefinition.html">Object2DDefinition</a></code> {#k-object_2d-name-definition}

<div class="api-description">

Declares a reusable 2D object; instantiate with Definition(options).
Define on_draw, on_bounds, on_update and ordinary functions inside or after the declaration.

<div class="api-arguments">

- <span class="api-parameter">name</span>: Object type name, such as "Meter".

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">object_3d</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">, </span><span class="api-parameter">definition</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object3DOptions.html">Object3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object3DDefinition.html">Object3DDefinition</a></code> {#k-object_3d-name-definition}

<div class="api-description">

Declares a reusable 3D object; instantiate with Definition(options).
Define on_draw, on_bounds, on_update and ordinary functions inside or after the declaration.

<div class="api-arguments">

- <span class="api-parameter">name</span>: Object type name, such as "Sculpture".

</div>

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
Use k.panel(options) to create a rectangular scene object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">box</span><span class="api-punctuation">(</span><span class="api-parameter">x</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">y</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">z</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">width</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">height</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">depth</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="BoxValue.html">BoxValue</a></code> {#k-box-x-y-z-width-height-depth}

<div class="api-description">

Creates immutable 3D bounds from local coordinates and nonnegative dimensions.

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

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">load_image</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image.html">Image</a></code> {#k-load_image-path}

<div class="api-description">

Live texture produced by a canvas or projection.
Loads an image resource from a path relative to the project directory.
Use the returned handle as an image object source or in a drawing callback.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">load_font</span><span class="api-punctuation">(</span><span class="api-parameter">path</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Font.html">Font</a></code> {#k-load_font-path}

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

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">circle</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="CircleOptions.html">CircleOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Circle.html">Circle</a></code> {#k-circle-options}

<div class="api-description">

Built-in circular scene object.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">ellipse</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="EllipseOptions.html">EllipseOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Ellipse.html">Ellipse</a></code> {#k-ellipse-options}

<div class="api-description">

Built-in elliptical scene object.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">panel</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PanelOptions.html">PanelOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Panel.html">Panel</a></code> {#k-panel-options}

<div class="api-description">

Built-in rectangular scene object.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">triangle</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="TriangleOptions.html">TriangleOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Triangle.html">Triangle</a></code> {#k-triangle-options}

<div class="api-description">

Built-in triangle scene object with an independently animatable apex.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">regular_polygon_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2DOptions.html">RegularPolygon2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2D.html">RegularPolygon2D</a></code> {#k-regular_polygon_2d-options}

<div class="api-description">

Built-in regular polygon scene object.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">line_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line2DOptions.html">Line2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line2D.html">Line2D</a></code> {#k-line_2d-options}

<div class="api-description">

Built-in line scene object with independently animatable arrowheads.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">image_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image2DOptions.html">Image2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image2D.html">Image2D</a></code> {#k-image_2d-options}

<div class="api-description">

Image drawn inside a styled rectangular shape.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">code</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeOptions.html">CodeOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Code.html">Code</a></code> {#k-code-options}

<div class="api-description">

Built-in 2D code object with syntax coloring, animated edits, and emphasis.

```lua
local code = k.code {language = "python", code = "print(42)"}
k.world_2d():add(code)
code:append("print(100)"):play(1)
```
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">text_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text2DOptions.html">Text2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text2D.html">Text2D</a></code> {#k-text_2d-options}

<div class="api-description">

Built-in text scene object.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">latex_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex2DOptions.html">Latex2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex2D.html">Latex2D</a></code> {#k-latex_2d-options}

<div class="api-description">

Built-in LaTeX math scene object, rendered with embedded KaTeX fonts.

Supports mathematical LaTeX accepted by RaTeX in display style.
Invalid or unsupported source panics when its geometry is first requested.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">group_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group2DOptions.html">Group2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group2D.html">Group2D</a></code> {#k-group_2d-options}

<div class="api-description">

Transformable scene object that groups an ordered set of child objects.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">container_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container2DOptions.html">Container2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container2D.html">Container2D</a></code> {#k-container_2d-options}

<div class="api-description">

A two-dimensional node that lays out its children procedurally.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">canvas_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas2DOptions.html">Canvas2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas2D.html">Canvas2D</a></code> {#k-canvas_2d-options}

<div class="api-description">

Independent 2D Skia viewport with its own camera.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">projection_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection2DOptions.html">Projection2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection2D.html">Projection2D</a></code> {#k-projection_2d-options}

<div class="api-description">

2D rectangle sampling the premultiplied output of a canvas.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">sphere</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="SphereOptions.html">SphereOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Sphere.html">Sphere</a></code> {#k-sphere-options}

<div class="api-description">

Sphere centered on its local origin.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">torus</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="TorusOptions.html">TorusOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Torus.html">Torus</a></code> {#k-torus-options}

<div class="api-description">

Torus centered on its local origin with its ring around the Y axis.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">prism</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions.html">PrismOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#k-prism-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">pyramid</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PyramidOptions.html">PyramidOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Pyramid.html">Pyramid</a></code> {#k-pyramid-options}

<div class="api-description">

Regular-base pyramid centered on its local origin with its apex facing positive Y.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">plane</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PlaneOptions.html">PlaneOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Plane.html">Plane</a></code> {#k-plane-options}

<div class="api-description">

Plane in local XY with its front facing positive Z.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">regular_polygon_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions.html">RegularPolygon3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#k-regular_polygon_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">line_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line3DOptions.html">Line3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line3D.html">Line3D</a></code> {#k-line_3d-options}

<div class="api-description">

Built-in 3D line scene object with independently animatable arrowheads.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">image_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image3DOptions.html">Image3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image3D.html">Image3D</a></code> {#k-image_3d-options}

<div class="api-description">

Textured plane in local XY with its front facing positive Z.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">text_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text3DOptions.html">Text3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text3D.html">Text3D</a></code> {#k-text_3d-options}

<div class="api-description">

Built-in extruded text scene object.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">latex_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex3DOptions.html">Latex3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex3D.html">Latex3D</a></code> {#k-latex_3d-options}

<div class="api-description">

Built-in extruded LaTeX math scene object.

Supports mathematical LaTeX accepted by RaTeX in display style.
Invalid or unsupported source panics when its geometry is first requested.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">group_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group3DOptions.html">Group3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group3D.html">Group3D</a></code> {#k-group_3d-options}

<div class="api-description">

Container inheriting three-dimensional transforms.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">container_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container3DOptions.html">Container3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container3D.html">Container3D</a></code> {#k-container_3d-options}

<div class="api-description">

A three-dimensional node that lays out its children procedurally.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">canvas_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas3DOptions.html">Canvas3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas3D.html">Canvas3D</a></code> {#k-canvas_3d-options}

<div class="api-description">

Independent 3D perspective viewport with its own camera.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">projection_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection3DOptions.html">Projection3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection3D.html">Projection3D</a></code> {#k-projection_3d-options}

<div class="api-description">

Unlit 3D plane sampling the premultiplied output of a canvas.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">directional_light</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="DirectionalLight3DOptions.html">DirectionalLight3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="DirectionalLight3D.html">DirectionalLight3D</a></code> {#k-directional_light-options}

<div class="api-description">

Light shining along its local negative Z axis.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">point_light</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PointLight3DOptions.html">PointLight3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="PointLight3D.html">PointLight3D</a></code> {#k-point_light-options}

<div class="api-description">

Light shining in every direction from its position.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">spot_light</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="SpotLight3DOptions.html">SpotLight3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="SpotLight3D.html">SpotLight3D</a></code> {#k-spot_light-options}

<div class="api-description">

Light shining in a cone along its local negative Z axis.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">square</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2DOptions_square.html">RegularPolygon2DOptions_square</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2D.html">RegularPolygon2D</a></code> {#k-square-options}

<div class="api-description">

Built-in regular polygon scene object.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">triangle_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions_triangle_3d.html">RegularPolygon3DOptions_triangle_3d</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#k-triangle_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">square_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions_square_3d.html">RegularPolygon3DOptions_square_3d</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#k-square_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">circle_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions_circle_3d.html">RegularPolygon3DOptions_circle_3d</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#k-circle_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">tetrahedron</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions_tetrahedron.html">PrismOptions_tetrahedron</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#k-tetrahedron-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">cube</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions_cube.html">PrismOptions_cube</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#k-cube-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">cylinder</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions_cylinder.html">PrismOptions_cylinder</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#k-cylinder-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="k.html">k</a><span class="api-punctuation">.</span><span class="api-function">cone</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="PyramidOptions_cone.html">PyramidOptions_cone</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Pyramid.html">Pyramid</a></code> {#k-cone-options}

<div class="api-description">

Regular-base pyramid centered on its local origin with its apex facing positive Y.
Creates an object in this scene; attach it to a group or canvas to activate it.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.

</div>

</div>

</section>

</div>

:::
