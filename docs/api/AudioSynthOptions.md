---
editLink: false
outline: 2
---

# AudioSynthOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Parameters for an immutable procedural audio resource; times are seconds and frequencies are Hz.

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">preset</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-string">&quot;boink&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;pluck&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;click&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;impact&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;whoosh&quot;</span></code> {#property-preset}

<div class="api-description">

Overridable starting parameters.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">waveform</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-string">&quot;sine&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;triangle&quot;</span> <span class="api-punctuation">|</span> <span class="api-string">&quot;noise&quot;</span></code> {#property-waveform}

<div class="api-description">

Default sine.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">frequency</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-frequency}

<div class="api-description">

Initial frequency, 1–20000; default 440.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">end_frequency</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-end_frequency}

<div class="api-description">

Final frequency of a linear sweep, 1–20000.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">attack</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-attack}

<div class="api-description">

Attack time, 0–10; default 0.004.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">decay</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-decay}

<div class="api-description">

Decay time, 0–10; default 0.08.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">sustain</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-sustain}

<div class="api-description">

Sustain level, 0–1; default 0.3.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">hold</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-hold}

<div class="api-description">

Time at sustain level, 0–10; default 0.03.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">release</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-release}

<div class="api-description">

Release time, 0–10; default 0.15.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">noise</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-noise}

<div class="api-description">

White noise blend, 0–1; default 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">overtone</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-overtone}

<div class="api-description">

Second sine harmonic gain, 0–1; default 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">cutoff</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span></code> {#property-cutoff}

<div class="api-description">

Low-pass cutoff, 20–20000; default 12000.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">seed</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span></code> {#property-seed}

<div class="api-description">

Deterministic noise seed, 0–4294967295; default 0.

</div>

</section>

</div>

</div>

:::
