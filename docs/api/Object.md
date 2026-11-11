---
editLink: false
outline: 2
---

# Object

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Scene object exposing properties, hierarchy operations, animations, and signals.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">get</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">any</span></code> {#object-get-name}

<div class="api-description">

Reads a named property value.
Names may include scalar channels, such as position.x. Use :prop to obtain a property handle instead.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">prop</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a></code> {#object-prop-name}

<div class="api-description">

Returns a named property handle, including names that collide with methods.
Accepts channels such as position.x; the handle supports get, set, animation, and signals.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">to</span><span class="api-punctuation">(</span><span class="api-parameter">fields</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#object-to-fields}

<div class="api-description">

Creates an animation toward absolute property destinations.
Accepts partial channels such as {position={x=100}}. Preparation leaves the timeline unchanged; call :play to schedule it.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">by</span><span class="api-punctuation">(</span><span class="api-parameter">fields</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#object-by-fields}

<div class="api-description">

Creates an animation adding relative property increments.
Omitted channels remain unchanged. Each :play resolves its starting values at its own scheduled time.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">from</span><span class="api-punctuation">(</span><span class="api-parameter">fields</span><span class="api-punctuation">: </span><span class="api-type-name">table</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#object-from-fields}

<div class="api-description">

Creates an animation with explicit start and finish pairs for each property or channel.
Example: object:from {opacity={0,1}}:play(1). Overlapping destinations are errors.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">add</span><span class="api-punctuation">(</span><span class="api-parameter">child</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">T</span></code> {#object-add-child}

<div class="api-description">

Attaches a child subtree at the current schedule time.
The receiver must support children, and both objects must belong to the same scene. Newly constructed objects stay inactive until attached.

**Type parameter:** <code class="api-signature"><span class="api-parameter">T</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Object.html">Object</a></code>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">reparent</span><span class="api-punctuation">(</span><span class="api-parameter">node</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">)</span></code> {#object-reparent-node}

<div class="api-description">

Moves this object to a new container for the whole timeline, preserving local properties and lifetime.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">remove</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#object-remove}

<div class="api-description">

Ends this object subtree lifetime at the current schedule time.
Entities and hierarchy edges are retained so seeking backward restores the object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">signal</span><span class="api-punctuation">(</span><span class="api-parameter">callback</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">object</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <span class="api-punctuation">{</span><span class="api-parameter">time</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">index</span><span class="api-punctuation">:</span> <span class="api-type-name">integer</span><span class="api-punctuation">,</span> <span class="api-parameter">dt</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Signal.html">Signal</a></code> {#object-signal-callback}

<div class="api-description">

Schedules a callback after scene evaluation while the object is active.
Receives the object and {time,index,dt}. Writes are temporary for that frame; the returned signal supports :stop().

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">snapshot</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Snapshot.html">Snapshot</a></code> {#object-snapshot}

<div class="api-description">

Captured object property values accepted by :restore.
Captures current tracked fields, dynamic properties, and shader uniforms.
Pass the returned snapshot to :restore(snapshot); it belongs to this object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">save</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#object-save}

<div class="api-description">

Pushes current property values onto this object snapshot stack.
A later :restore() without an argument pops the saved values.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">restore</span><span class="api-punctuation">(</span><span class="api-parameter">snapshot</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Snapshot.html">Snapshot</a><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#object-restore-snapshot}

<div class="api-description">

Creates an animation toward an explicit snapshot, or pops the last :save() snapshot.
The snapshot must belong to this object. Call :play to schedule the restoration.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">reset</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#object-reset}

<div class="api-description">

Creates an animation toward the initial property values.
Call :play to schedule the reset.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">children</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#object-children}

<div class="api-description">

Returns child object handles in hierarchy order.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">parent</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">?</span></code> {#object-parent}

<div class="api-description">

Returns the parent object, or nil when there is no parent.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">ancestor</span><span class="api-punctuation">(</span><span class="api-parameter">type_name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">?</span></code> {#object-ancestor-type_name}

<div class="api-description">

Returns the first ancestor whose :type_name() matches the supplied name.
Returns nil when no ancestor matches.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">type_name</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#object-type_name}

<div class="api-description">

Returns the native object type name or the configured name of a scripted object.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">get_child</span><span class="api-punctuation">(</span><span class="api-parameter">index</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a></code> {#object-get_child-index}

<div class="api-description">

Returns the child at a one-based position in hierarchy order.
The index must refer to an existing child.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">insert</span><span class="api-punctuation">(</span><span class="api-parameter">child</span><span class="api-punctuation">: </span><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">, </span><span class="api-parameter">index</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">)</span></code> {#object-insert-child-index}

<div class="api-description">

Attaches a child subtree at the specified one-based hierarchy position.
Both objects must belong to the same scene and the receiver must support children.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">remove_children</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#object-remove_children}

<div class="api-description">

Ends the lifetimes of the child subtrees at the current schedule time.
Seeking backward restores their earlier visibility.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">move_to</span><span class="api-punctuation">(</span><span class="api-parameter">index</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">)</span></code> {#object-move_to-index}

<div class="api-description">

Moves this object to a one-based position among its siblings.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">move_up</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#object-move_up}

<div class="api-description">

Moves this object one position toward the end of its sibling list.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">move_down</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#object-move_down}

<div class="api-description">

Moves this object one position toward the start of its sibling list.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">move_to_top</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#object-move_to_top}

<div class="api-description">

Moves this object to the end of its sibling list, above its siblings.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">move_to_bottom</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span></code> {#object-move_to_bottom}

<div class="api-description">

Moves this object to the start of its sibling list, below its siblings.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">on_bounds</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="RectValue.html">RectValue</a> <span class="api-punctuation">|</span> <a class="api-type-name" href="BoxValue.html">BoxValue</a></code> {#object-on_bounds}

<div class="api-description">

Returns the local bounds using evaluated properties and simulation state.
This is also the callback implemented by custom objects.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">global_matrix</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Matrix.html">Matrix</a></code> {#object-global_matrix}

<div class="api-description">

Returns the transformation matrix composed through the object hierarchy.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Object.html">Object</a><span class="api-punctuation">:</span><span class="api-function">uniform</span><span class="api-punctuation">(</span><span class="api-parameter">name</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a></code> {#object-uniform-name}

<div class="api-description">

Returns a shader uniform property handle by name.
The uniform must be configured on the object.

</div>

</section>

</div>

:::
