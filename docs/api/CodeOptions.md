---
editLink: false
outline: 2
---

# CodeOptions

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Initial properties and resources for code.

**Inherits:** <code class="api-signature"><a class="api-type-name" href="ResourceOptions.html">ResourceOptions</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">id</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#property-id}

<div class="api-description">

Stable identifier used to preserve object identity across reloads.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">name</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span></code> {#property-name}

<div class="api-description">

Display name in the scene tree.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">code</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation"> = </span><span class="api-string">&quot;&quot;</span></code> {#property-code}

<div class="api-description">

Code displayed by the object, preserving indentation.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">size</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">36</span></code> {#property-size}

<div class="api-description">

Font size in logical canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">line_spacing</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-line_spacing}

<div class="api-description">

Line spacing as a multiple of the font's natural spacing. Minimum: 0.1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">tab_size</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation"> = </span><span class="api-literal">4</span></code> {#property-tab_size}

<div class="api-description">

Number of visual columns between tab stops. Minimum: 1. Maximum: 16.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">language</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-string">&#x27;lua&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;plain&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;javascript&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;typescript&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;jsx&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;tsx&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;html&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;css&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;scss&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;python&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;c&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;cpp&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;csharp&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;java&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;rust&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;go&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;php&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;ruby&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;swift&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;kotlin&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;dart&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;bash&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;powershell&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;sql&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;r&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;json&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;jsonc&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;yaml&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;toml&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;xml&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;ini&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;markdown&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;latex&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;dockerfile&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;makefile&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;cmake&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;glsl&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;hlsl&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;wgsl&#x27;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;plain&quot;</span></code> {#property-language}

<div class="api-description">

Syntax language, defaulting to plain. Web: javascript, typescript, jsx, tsx, html, css, scss. General: python, c, cpp, csharp, java, rust, go, php, ruby, swift, kotlin, dart. Scripts and data: bash, powershell, sql, r. Configuration: json, jsonc, yaml, toml, xml, ini. Documents: markdown, latex. Build: dockerfile, makefile, cmake. Shaders: glsl, hlsl, wgsl.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">keyword_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.78</span><span class="api-punctuation">,</span> <span class="api-literal">0.47</span><span class="api-punctuation">,</span> <span class="api-literal">0.87</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-keyword_color}

<div class="api-description">

Color of keywords, types, and language-specific declarations.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">string_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.6</span><span class="api-punctuation">,</span> <span class="api-literal">0.76</span><span class="api-punctuation">,</span> <span class="api-literal">0.47</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-string_color}

<div class="api-description">

Color of string literals and quoted values.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">number_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.82</span><span class="api-punctuation">,</span> <span class="api-literal">0.6</span><span class="api-punctuation">,</span> <span class="api-literal">0.4</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-number_color}

<div class="api-description">

Color of numeric literals.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">comment_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.45</span><span class="api-punctuation">,</span> <span class="api-literal">0.51</span><span class="api-punctuation">,</span> <span class="api-literal">0.56</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-comment_color}

<div class="api-description">

Color of line and long bracket comments.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">operator_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.34</span><span class="api-punctuation">,</span> <span class="api-literal">0.71</span><span class="api-punctuation">,</span> <span class="api-literal">0.76</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-operator_color}

<div class="api-description">

Color of operators and punctuation.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">highlight_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.3</span><span class="api-punctuation">,</span> <span class="api-literal">0.55</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">0.3</span><span class="api-punctuation">}</span></code> {#property-highlight_color}

<div class="api-description">

Background color of highlighted code ranges.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">dim_opacity</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0.25</span></code> {#property-dim_opacity}

<div class="api-description">

Opacity multiplier for text outside the selection. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">selection</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeRange.html">CodeRange</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#property-selection}

<div class="api-description">

Selected half-open ranges, using one-based Unicode line and column positions. Default: [] (1.00).

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">highlights</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeRange.html">CodeRange</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#property-highlights}

<div class="api-description">

Ranges marked with a colored background, independent of selection. Default: [] (1.00).

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">fill</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-fill}

<div class="api-description">

Color used to fill the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-stroke}

<div class="api-description">

Color used to outline the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke_width</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-stroke_width}

<div class="api-description">

Width used to outline the style. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">position</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-position}

<div class="api-description">

Position of the entity.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">origin</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-origin}

<div class="api-description">

Pivot relative to half the entity size. Values are not clamped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">scale</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-scale}

<div class="api-description">

Scale of the entity on each axis.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">rotation</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-rotation}

<div class="api-description">

Rotation of the entity in radians.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">opacity</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-opacity}

<div class="api-description">

Transparency applied while drawing this entity, from `0` to `1`. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">z_index</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-z_index}

<div class="api-description">

Stacking order among sibling objects. Higher values are drawn in front.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">follows_camera</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-follows_camera}

<div class="api-description">

Whether this entity and its subtree are affected by the canvas camera.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">blur</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-blur}

<div class="api-description">

Gaussian blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_offset</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2.html">Vec2</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">x</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">y</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_offset}

<div class="api-description">

Shadow offset in local canvas units.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_blur</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><span class="api-type-name">number</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-shadow_blur}

<div class="api-description">

Shadow blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_color</span><span class="api-punctuation">?</span><span class="api-punctuation">: </span><a class="api-type-name" href="Color.html">Color</a> <span class="api-punctuation">|</span> <span class="api-punctuation">{</span><span class="api-parameter">r</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">g</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">b</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">,</span> <span class="api-parameter">a</span><span class="api-punctuation">?</span><span class="api-punctuation">:</span> <span class="api-type-name">number</span><span class="api-punctuation">}</span><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_color}

<div class="api-description">

Shadow color.

</div>

</section>

</div>

</div>

:::
