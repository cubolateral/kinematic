---
editLink: false
outline: 2
---

# Scene

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Scene construction and animation scheduling context passed to a scene function.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">audio</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="AudioOptions.html">AudioOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Audio.html">Audio</a></code> {#scene-audio-options}

<div class="api-description">

Creates a silent player. Scheduled tails contribute to scene duration; repeat scopes are rejected.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">view</span><span class="api-punctuation">(</span><span class="api-parameter">dimension</span><span class="api-punctuation">: </span><span class="api-string">&quot;2d&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;3d&quot;</span><span class="api-punctuation">)</span></code> {#scene-view-dimension}

<div class="api-description">

Schedules an instantaneous switch between the built-in 2D and 3D world canvases.
The dimension must be "2d" or "3d".

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">wait</span><span class="api-punctuation">(</span><span class="api-parameter">seconds</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span></code> {#scene-wait-seconds}

<div class="api-description">

Advances the current scheduling scope by a nonnegative duration in seconds.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">event</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span></code> {#scene-event-name}

<div class="api-description">

Adds a named editable wait at the current timeline position.
Its duration can be adjusted in the editor.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">all</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">scene</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#scene-all-schedule}

<div class="api-description">

Schedules the callback animations simultaneously.
The parent timeline advances by the longest child duration.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">chain</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">scene</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#scene-chain-schedule}

<div class="api-description">

Schedules the callback animations sequentially on the current timeline.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">parallel</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">scene</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#scene-parallel-schedule}

<div class="api-description">

Schedules the callback without advancing or extending the parent timeline.
Animations inside the callback are sequential unless grouped with :all.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">loop</span><span class="api-punctuation">(</span><span class="api-parameter">schedule</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">scene</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span></code> {#scene-loop-schedule}

<div class="api-description">

Repeats one positive finite animation cycle without advancing the parent timeline.
The callback executes once; its animations repeat until their lifetime ends. Nested loops and object creation, attachment, or removal within a cycle are rejected.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">world_2d</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas2D.html">Canvas2D</a></code> {#scene-world_2d}

<div class="api-description">

Returns the built-in 2D canvas.
Attach 2D objects with scene:world_2d():add(object).

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">world_3d</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas3D.html">Canvas3D</a></code> {#scene-world_3d}

<div class="api-description">

Returns the built-in 3D canvas.
Attach 3D objects with scene:world_3d():add(object).

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">root</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a></code> {#scene-root}

<div class="api-description">

Returns the internal root container shared by all scene contents.
Attach additional canvases here; use :world_2d() and :world_3d() for ordinary world objects.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">track</span><span class="api-punctuation">(</span><span class="api-parameter">initial</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">, </span><span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">T</span><span class="api-punctuation">&gt;</span></code> {#scene-track-initial-id}

<div class="api-description">

Creates a standalone animatable property from an initial value.
An optional stable id preserves object identity across reloads. Supports get, set, to, by, from, and signal.

**Type parameter:** <code class="api-signature"><span class="api-type-name">T</span></code>

**Overloads**

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">,</span> <span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2.html">Vec2</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec2Property.html">Vec2Property</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">,</span> <span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec3.html">Vec3</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Vec3Property.html">Vec3Property</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">,</span> <span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Color.html">Color</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="ColorProperty.html">ColorProperty</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">,</span> <span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Quaternion.html">Quaternion</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="QuaternionProperty.html">QuaternionProperty</a></code>

<code class="api-signature"><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">,</span> <span class="api-parameter">initial</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Quad.html">Quad</a><span class="api-punctuation">,</span> <span class="api-parameter">id</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="QuadProperty.html">QuadProperty</a></code>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">object</span><span class="api-punctuation">(</span><span class="api-parameter">definition</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">, </span><span class="api-parameter">options</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">T</span></code> {#scene-object-definition-options}

<div class="api-description">

Instantiates a k.object definition with initial property and resource options.
Each instance receives independent state and stays inactive until attached to a group or canvas.

**Type parameter:** <code class="api-signature"><span class="api-type-name">T</span></code>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">circle</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="CircleOptions.html">CircleOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Circle.html">Circle</a></code> {#scene-circle-options}

<div class="api-description">

Built-in circular scene object.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">ellipse</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="EllipseOptions.html">EllipseOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Ellipse.html">Ellipse</a></code> {#scene-ellipse-options}

<div class="api-description">

Built-in elliptical scene object.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">rect</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="RectOptions.html">RectOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Rect.html">Rect</a></code> {#scene-rect-options}

<div class="api-description">

Built-in rectangular scene object.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">triangle</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="TriangleOptions.html">TriangleOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Triangle.html">Triangle</a></code> {#scene-triangle-options}

<div class="api-description">

Built-in triangle scene object with an independently animatable apex.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">regular_polygon_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2DOptions.html">RegularPolygon2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2D.html">RegularPolygon2D</a></code> {#scene-regular_polygon_2d-options}

<div class="api-description">

Built-in regular polygon scene object.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">line_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line2DOptions.html">Line2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line2D.html">Line2D</a></code> {#scene-line_2d-options}

<div class="api-description">

Built-in line scene object with independently animatable arrowheads.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">image_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image2DOptions.html">Image2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image2D.html">Image2D</a></code> {#scene-image_2d-options}

<div class="api-description">

Image drawn inside a styled rectangular shape.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">code</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeOptions.html">CodeOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Code.html">Code</a></code> {#scene-code-options}

<div class="api-description">

Built-in 2D code object with syntax coloring, animated edits, and emphasis.

```lua
local code = s:code {language = "python", code = "print(42)"}
s:world_2d():add(code)
code:append("print(100)"):play(1)
```
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">text_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text2DOptions.html">Text2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text2D.html">Text2D</a></code> {#scene-text_2d-options}

<div class="api-description">

Built-in text scene object.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">latex_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex2DOptions.html">Latex2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex2D.html">Latex2D</a></code> {#scene-latex_2d-options}

<div class="api-description">

Built-in LaTeX math scene object, rendered with embedded KaTeX fonts.

Supports mathematical LaTeX accepted by RaTeX in display style.
Invalid or unsupported source panics when its geometry is first requested.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">group_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group2DOptions.html">Group2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group2D.html">Group2D</a></code> {#scene-group_2d-options}

<div class="api-description">

Transformable scene object that groups an ordered set of child objects.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">container_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container2DOptions.html">Container2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container2D.html">Container2D</a></code> {#scene-container_2d-options}

<div class="api-description">

A two-dimensional node that lays out its children procedurally.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">canvas_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas2DOptions.html">Canvas2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas2D.html">Canvas2D</a></code> {#scene-canvas_2d-options}

<div class="api-description">

Independent 2D Skia viewport with its own camera.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">projection_2d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection2DOptions.html">Projection2DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection2D.html">Projection2D</a></code> {#scene-projection_2d-options}

<div class="api-description">

2D rectangle sampling the premultiplied output of a canvas.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">sphere</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="SphereOptions.html">SphereOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Sphere.html">Sphere</a></code> {#scene-sphere-options}

<div class="api-description">

Sphere centered on its local origin.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">torus</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="TorusOptions.html">TorusOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Torus.html">Torus</a></code> {#scene-torus-options}

<div class="api-description">

Torus centered on its local origin with its ring around the Y axis.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">prism</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions.html">PrismOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#scene-prism-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">pyramid</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PyramidOptions.html">PyramidOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Pyramid.html">Pyramid</a></code> {#scene-pyramid-options}

<div class="api-description">

Regular-base pyramid centered on its local origin with its apex facing positive Y.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">plane</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PlaneOptions.html">PlaneOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Plane.html">Plane</a></code> {#scene-plane-options}

<div class="api-description">

Plane in local XY with its front facing positive Z.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">regular_polygon_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions.html">RegularPolygon3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#scene-regular_polygon_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">line_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line3DOptions.html">Line3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Line3D.html">Line3D</a></code> {#scene-line_3d-options}

<div class="api-description">

Built-in 3D line scene object with independently animatable arrowheads.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">image_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image3DOptions.html">Image3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Image3D.html">Image3D</a></code> {#scene-image_3d-options}

<div class="api-description">

Textured plane in local XY with its front facing positive Z.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">text_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text3DOptions.html">Text3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Text3D.html">Text3D</a></code> {#scene-text_3d-options}

<div class="api-description">

Built-in extruded text scene object.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">latex_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex3DOptions.html">Latex3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Latex3D.html">Latex3D</a></code> {#scene-latex_3d-options}

<div class="api-description">

Built-in extruded LaTeX math scene object.

Supports mathematical LaTeX accepted by RaTeX in display style.
Invalid or unsupported source panics when its geometry is first requested.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">group_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group3DOptions.html">Group3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Group3D.html">Group3D</a></code> {#scene-group_3d-options}

<div class="api-description">

Container inheriting three-dimensional transforms.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">container_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container3DOptions.html">Container3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Container3D.html">Container3D</a></code> {#scene-container_3d-options}

<div class="api-description">

A three-dimensional node that lays out its children procedurally.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">canvas_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas3DOptions.html">Canvas3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Canvas3D.html">Canvas3D</a></code> {#scene-canvas_3d-options}

<div class="api-description">

Independent 3D perspective viewport with its own camera.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">projection_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection3DOptions.html">Projection3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Projection3D.html">Projection3D</a></code> {#scene-projection_3d-options}

<div class="api-description">

Unlit 3D plane sampling the premultiplied output of a canvas.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">directional_light</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="DirectionalLight3DOptions.html">DirectionalLight3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="DirectionalLight3D.html">DirectionalLight3D</a></code> {#scene-directional_light-options}

<div class="api-description">

Light shining along its local negative Z axis.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">point_light</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PointLight3DOptions.html">PointLight3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="PointLight3D.html">PointLight3D</a></code> {#scene-point_light-options}

<div class="api-description">

Light shining in every direction from its position.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">spot_light</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="SpotLight3DOptions.html">SpotLight3DOptions</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="SpotLight3D.html">SpotLight3D</a></code> {#scene-spot_light-options}

<div class="api-description">

Light shining in a cone along its local negative Z axis.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">square</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2DOptions_square.html">RegularPolygon2DOptions_square</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon2D.html">RegularPolygon2D</a></code> {#scene-square-options}

<div class="api-description">

Built-in regular polygon scene object.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">triangle_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions_triangle_3d.html">RegularPolygon3DOptions_triangle_3d</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#scene-triangle_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">square_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions_square_3d.html">RegularPolygon3DOptions_square_3d</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#scene-square_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">circle_3d</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3DOptions_circle_3d.html">RegularPolygon3DOptions_circle_3d</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RegularPolygon3D.html">RegularPolygon3D</a></code> {#scene-circle_3d-options}

<div class="api-description">

Flat regular polygon with its front facing positive Z.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">tetrahedron</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions_tetrahedron.html">PrismOptions_tetrahedron</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#scene-tetrahedron-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">cube</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions_cube.html">PrismOptions_cube</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#scene-cube-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">cylinder</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PrismOptions_cylinder.html">PrismOptions_cylinder</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Prism.html">Prism</a></code> {#scene-cylinder-options}

<div class="api-description">

Regular-base prism centered on its local origin.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Scene.html">Scene</a><span class="api-punctuation">:</span><span class="api-function">cone</span><span class="api-punctuation">(</span><span class="api-parameter">options</span><span class="api-punctuation">: </span><a class="api-type-name" href="PyramidOptions_cone.html">PyramidOptions_cone</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Pyramid.html">Pyramid</a></code> {#scene-cone-options}

<div class="api-description">

Regular-base pyramid centered on its local origin with its apex facing positive Y.
Creates the object with the supplied initial properties at the current schedule time.

<div class="api-arguments">

- <span class="api-parameter">options</span>: Initial properties, name, identifier, and resources.
- **Returns:** object Handle exposing animatable properties and object methods.

</div>

</div>

</section>

</div>

:::
