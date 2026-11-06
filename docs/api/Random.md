---
editLink: false
outline: 2
---

# Random

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Deterministic mutable random generator that can be preserved in simulation state.

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">number</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#random-number}

<div class="api-description">

Returns a uniformly distributed number from 0 inclusive to 1 exclusive.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">integer</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span></code> {#random-integer}

<div class="api-description">

Returns a uniformly distributed unsigned 32-bit integer.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">boolean</span><span class="api-punctuation">(</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span></code> {#random-boolean}

<div class="api-description">

Returns a random boolean with equal probability for each value.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">fork</span><span class="api-punctuation">(</span><span class="api-parameter">key</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Random.html">Random</a></code> {#random-fork-key}

<div class="api-description">

Creates an independent deterministic generator derived from this seed and key.
Does not consume the parent generator sequence.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">range</span><span class="api-punctuation">(</span><span class="api-parameter">min</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">max</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#random-range-min-max}

<div class="api-description">

Returns a uniformly distributed number in [min, max).
Both bounds must be finite and min must be less than max.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">range_integer</span><span class="api-punctuation">(</span><span class="api-parameter">min</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">, </span><span class="api-parameter">max</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span></code> {#random-range_integer-min-max}

<div class="api-description">

Returns a uniformly distributed integer in [min, max).
The minimum must be less than the exclusive maximum.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">chance</span><span class="api-punctuation">(</span><span class="api-parameter">probability</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span></code> {#random-chance-probability}

<div class="api-description">

Returns true with the supplied probability from 0 to 1 inclusive.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">normal</span><span class="api-punctuation">(</span><span class="api-parameter">mean</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">, </span><span class="api-parameter">deviation</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#random-normal-mean-deviation}

<div class="api-description">

Returns a normally distributed number with the supplied mean and standard deviation.

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">choose</span><span class="api-punctuation">(</span><span class="api-parameter">values</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">, </span><span class="api-parameter">weights</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">?</span></code> {#random-choose-values-weights}

<div class="api-description">

Chooses one array entry, optionally using matching nonnegative weights.
Returns nil for an empty array, invalid weights, or all-zero weights.

**Type parameter:** <code class="api-signature"><span class="api-type-name">T</span></code>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">choose_multiple</span><span class="api-punctuation">(</span><span class="api-parameter">values</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">, </span><span class="api-parameter">amount</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#random-choose_multiple-values-amount}

<div class="api-description">

Chooses up to amount distinct array entries without changing the input array.

**Type parameter:** <code class="api-signature"><span class="api-type-name">T</span></code>

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Random.html">Random</a><span class="api-punctuation">:</span><span class="api-function">shuffle</span><span class="api-punctuation">(</span><span class="api-parameter">values</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><span class="api-type-name">T</span><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#random-shuffle-values}

<div class="api-description">

Randomly permutes the array in place and returns the same table.

**Type parameter:** <code class="api-signature"><span class="api-type-name">T</span></code>

</div>

</section>

</div>

:::
