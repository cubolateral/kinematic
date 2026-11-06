---
editLink: false
outline: 2
---

# Code

::: v-pre

<div class="api-reference">

[API index](index.md) · [Guides](../guide/start.md) · [Examples](../examples/index.md)

Source: [shared declarations](https://github.com/cubolateral/kinematic/blob/main/src/scripting/api.lua), [properties](https://github.com/cubolateral/kinematic/blob/main/src/scripting/property.lua) and [Rust metadata](https://github.com/cubolateral/kinematic/tree/main/src/core).

Built-in 2D code object with syntax coloring, animated edits, and emphasis.

```lua
local code = s:code {language = "python", code = "print(42)"}
s:world_2d():add(code)
code:append("print(100)"):play(1)
```

**Inherits:** <code class="api-signature"><a class="api-type-name" href="Spatial2D.html">Spatial2D</a></code>

## Properties

<div class="api-members">

<section class="api-member">

### <code class="api-signature"><span class="api-name">signal</span><span class="api-punctuation">: </span><span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">self</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">,</span> <span class="api-parameter">callback</span><span class="api-punctuation">:</span> <span class="api-keyword">fun</span><span class="api-punctuation">(</span><span class="api-parameter">object</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">,</span> <span class="api-parameter">frame</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Frame.html">Frame</a><span class="api-punctuation">)</span><span class="api-punctuation">)</span><span class="api-punctuation">:</span> <a class="api-type-name" href="Signal.html">Signal</a></code> {#property-signal}

<div class="api-description">

Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">code</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">string</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;&quot;</span></code> {#property-code}

<div class="api-description">

Code displayed by the object, preserving indentation.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">size</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">36</span></code> {#property-size}

<div class="api-description">

Font size in logical canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">line_spacing</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-line_spacing}

<div class="api-description">

Line spacing as a multiple of the font's natural spacing. Minimum: 0.1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">tab_size</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">integer</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">4</span></code> {#property-tab_size}

<div class="api-description">

Number of visual columns between tab stops. Minimum: 1. Maximum: 16.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">language</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-string">&#x27;lua&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;plain&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;javascript&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;typescript&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;jsx&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;tsx&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;html&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;css&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;scss&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;python&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;c&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;cpp&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;csharp&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;java&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;rust&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;go&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;php&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;ruby&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;swift&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;kotlin&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;dart&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;bash&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;powershell&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;sql&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;r&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;json&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;jsonc&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;yaml&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;toml&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;xml&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;ini&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;markdown&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;latex&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;dockerfile&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;makefile&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;cmake&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;glsl&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;hlsl&#x27;</span> <span class="api-punctuation">|</span> <span class="api-string">&#x27;wgsl&#x27;</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-string">&quot;plain&quot;</span></code> {#property-language}

<div class="api-description">

Syntax language, defaulting to plain. Web: javascript, typescript, jsx, tsx, html, css, scss. General: python, c, cpp, csharp, java, rust, go, php, ruby, swift, kotlin, dart. Scripts and data: bash, powershell, sql, r. Configuration: json, jsonc, yaml, toml, xml, ini. Documents: markdown, latex. Build: dockerfile, makefile, cmake. Shaders: glsl, hlsl, wgsl.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">keyword_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.78</span><span class="api-punctuation">,</span> <span class="api-literal">0.47</span><span class="api-punctuation">,</span> <span class="api-literal">0.87</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-keyword_color}

<div class="api-description">

Color of keywords, types, and language-specific declarations.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">string_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.6</span><span class="api-punctuation">,</span> <span class="api-literal">0.76</span><span class="api-punctuation">,</span> <span class="api-literal">0.47</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-string_color}

<div class="api-description">

Color of string literals and quoted values.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">number_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.82</span><span class="api-punctuation">,</span> <span class="api-literal">0.6</span><span class="api-punctuation">,</span> <span class="api-literal">0.4</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-number_color}

<div class="api-description">

Color of numeric literals.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">comment_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.45</span><span class="api-punctuation">,</span> <span class="api-literal">0.51</span><span class="api-punctuation">,</span> <span class="api-literal">0.56</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-comment_color}

<div class="api-description">

Color of line and long bracket comments.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">operator_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.34</span><span class="api-punctuation">,</span> <span class="api-literal">0.71</span><span class="api-punctuation">,</span> <span class="api-literal">0.76</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-operator_color}

<div class="api-description">

Color of operators and punctuation.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">highlight_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0.3</span><span class="api-punctuation">,</span> <span class="api-literal">0.55</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">0.3</span><span class="api-punctuation">}</span></code> {#property-highlight_color}

<div class="api-description">

Background color of highlighted code ranges.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">dim_opacity</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0.25</span></code> {#property-dim_opacity}

<div class="api-description">

Opacity multiplier for text outside the selection. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">selection</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><a class="api-type-name" href="CodeRange.html">CodeRange</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">&gt;</span></code> {#property-selection}

<div class="api-description">

Selected half-open ranges, using one-based Unicode line and column positions. Default: [] (1.00).

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">highlights</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><a class="api-type-name" href="CodeRange.html">CodeRange</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span><span class="api-punctuation">&gt;</span></code> {#property-highlights}

<div class="api-description">

Ranges marked with a colored background, independent of selection. Default: [] (1.00).

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">fill</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-fill}

<div class="api-description">

Color used to fill the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-stroke}

<div class="api-description">

Color used to outline the style.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">stroke_width</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-stroke_width}

<div class="api-description">

Width used to outline the style. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">position</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-position}

<div class="api-description">

Position of the entity.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">origin</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-origin}

<div class="api-description">

Pivot relative to half the entity size. Values are not clamped.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">scale</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">1</span><span class="api-punctuation">,</span> <span class="api-literal">1</span><span class="api-punctuation">}</span></code> {#property-scale}

<div class="api-description">

Scale of the entity on each axis.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">rotation</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-rotation}

<div class="api-description">

Rotation of the entity in radians.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">visibility</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-visibility}

<div class="api-description">

Whether this entity and, for containers, its subtree are drawn.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">opacity</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">1</span></code> {#property-opacity}

<div class="api-description">

Transparency applied while drawing this entity, from `0` to `1`. Minimum: 0. Maximum: 1.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">z_index</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">integer</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-z_index}

<div class="api-description">

Stacking order among sibling objects. Higher values are drawn in front.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">follows_camera</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">boolean</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">true</span></code> {#property-follows_camera}

<div class="api-description">

Whether this entity and its subtree are affected by the canvas camera.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">blur</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-blur}

<div class="api-description">

Gaussian blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_offset</span><span class="api-punctuation">: </span><a class="api-type-name" href="Vec2Property.html">Vec2Property</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_offset}

<div class="api-description">

Shadow offset in local canvas units.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_blur</span><span class="api-punctuation">: </span><a class="api-type-name" href="Property.html">Property</a><span class="api-punctuation">&lt;</span><span class="api-type-name">number</span><span class="api-punctuation">&gt;</span><span class="api-punctuation"> = </span><span class="api-literal">0</span></code> {#property-shadow_blur}

<div class="api-description">

Shadow blur sigma in local canvas units. Minimum: 0.

</div>

</section>

<section class="api-member">

### <code class="api-signature"><span class="api-name">shadow_color</span><span class="api-punctuation">: </span><a class="api-type-name" href="ColorProperty.html">ColorProperty</a><span class="api-punctuation"> = </span><span class="api-punctuation">{</span><span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">,</span> <span class="api-literal">0</span><span class="api-punctuation">}</span></code> {#property-shadow_color}

<div class="api-description">

Shadow color.

</div>

</section>

</div>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">append</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-append-text}

<div class="api-description">

Appends code on a new last line. Adds a line break when needed and fades in the entire addition, preserving existing tokens.

```lua
code:append('print(value)'):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">prepend</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-prepend-text}

<div class="api-description">

Prepends code on a new first line. Adds a line break when needed and fades in the entire addition, preserving existing tokens.

```lua
code:prepend('local value = 42'):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">prepand</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-prepand-text}

<div class="api-description">

Alias of prepend, adding code on a new first line with a fade.

```lua
code:prepand('local value = 42'):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">word</span><span class="api-punctuation">(</span><span class="api-parameter">line</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">, </span><span class="api-parameter">column</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">, </span><span class="api-parameter">length</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span> <span class="api-punctuation">|</span> <span class="api-literal">nil</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeRange.html">CodeRange</a></code> {#code-word-line-column-length}

<div class="api-description">

Selects characters on one line: word(line, column, length). Line and column start at 1. Omit length to select through the end of the line. The result can be passed to select, highlight, replace, or erase.

```lua
-- Three characters starting at column 3 of line 1.
code:select(code:word(1, 3, 3)):play(1)
-- Column 3 through the end of line 1.
code:highlight(code:word(1, 3)):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">lines</span><span class="api-punctuation">(</span><span class="api-parameter">first</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span><span class="api-punctuation">, </span><span class="api-parameter">last</span><span class="api-punctuation">: </span><span class="api-type-name">integer</span> <span class="api-punctuation">|</span> <span class="api-literal">nil</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeRange.html">CodeRange</a></code> {#code-lines-first-last}

<div class="api-description">

Selects whole lines: lines(first, last). Both line numbers start at 1 and are included. Omit last to select one line. The result can be passed to select, highlight, replace, or erase.

```lua
code:select(code:lines(1, 3)):play(1)
code:highlight(code:lines(2)):play(1)
code:select({code:find('1'), code:find_all('oi'), code:lines(1)}):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">morph</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">, </span><span class="api-parameter">group</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation">?</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-morph-text-group}

<div class="api-description">

Moves matching characters and fades inserted or removed characters. Group matches whole words and individual symbols instead. Default: false. Preparing leaves the timeline unchanged.

```lua
code:morph('local value = 42\nprint(value)'):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">fade</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-fade-text}

<div class="api-description">

Crossfades the current code into a new block.

```lua
code:fade('print(42)'):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">insert</span><span class="api-punctuation">(</span><span class="api-parameter">position</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodePosition.html">CodePosition</a><span class="api-punctuation">, </span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-insert-position-text}

<div class="api-description">

Inserts text at a one-based {line, column} position. Affected tokens fade as whole words; intact tokens keep their identity. Use morph for letter matching. Columns count Unicode characters, including each tab as one character. The position is checked against the code at each play.

```lua
-- Inserts before the first character of the second line.
code:insert({2, 1}, '-- Result.\n'):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">replace</span><span class="api-punctuation">(</span><span class="api-parameter">ranges</span><span class="api-punctuation">: </span><span class="api-type-name">CodeRanges</span> <span class="api-punctuation">|</span> <span class="api-literal">nil</span><span class="api-punctuation">, </span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-replace-ranges-text}

<div class="api-description">

Replaces one or several text intervals with the same text. Use find, find_all, word, or lines to choose the intervals; their results can be mixed in a list. Changed tokens fade as whole words. Edits use the code at play time. Overlapping intervals or a missing single search result raise an error.

```lua
-- Replaces the first occurrence.
code:replace(code:find('value'), 'answer'):play(1)
-- Replaces every occurrence of either name.
code:replace({code:find_all('foo'), code:find_all('bar')}, 'value'):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">erase</span><span class="api-punctuation">(</span><span class="api-parameter">ranges</span><span class="api-punctuation">: </span><span class="api-type-name">CodeRanges</span> <span class="api-punctuation">|</span> <span class="api-literal">nil</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-erase-ranges}

<div class="api-description">

Erases the text chosen by find, find_all, word, or lines. Accepts one interval or a list mixing their results. Changed tokens fade as whole words. Overlapping intervals or a missing single search result raise an error. Use remove to remove the scene object.

```lua
-- Removes the first line.
code:erase(code:lines(1)):play(1)
-- Removes every occurrence.
code:erase(code:find_all('debug')):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">select</span><span class="api-punctuation">(</span><span class="api-parameter">ranges</span><span class="api-punctuation">: </span><span class="api-type-name">CodeRanges</span> <span class="api-punctuation">|</span> <span class="api-literal">nil</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-select-ranges}

<div class="api-description">

Dims the text outside the chosen intervals, preserving syntax colors. Choose text with find, find_all, word, or lines, and optionally mix their results in a list. Nil or {} restores all text. Missing searches inside a list are ignored. Intervals keep their coordinates after edits.

```lua
code:select(code:find_all('value')):play(1)
-- Combines searches and a complete line.
code:select({code:find('1'), code:find_all('oi'), code:lines(1)}):play(1)
code:select(nil):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">highlight</span><span class="api-punctuation">(</span><span class="api-parameter">ranges</span><span class="api-punctuation">: </span><span class="api-type-name">CodeRanges</span> <span class="api-punctuation">|</span> <span class="api-literal">nil</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="Animation.html">Animation</a></code> {#code-highlight-ranges}

<div class="api-description">

Marks the chosen text with highlight_color. Choose text with find, find_all, word, or lines, and optionally mix their results in a list. Nil or {} removes the backgrounds. Missing searches inside a list are ignored. Included in exports. Intervals keep their coordinates after edits.

```lua
code:highlight(code:find_all('value')):play(1)
code:to {highlight_color = k.color('#FFCC0066')}:play(1)
code:highlight({}):play(1)
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">find</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">, </span><span class="api-parameter">match_case</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation">?</span><span class="api-punctuation">, </span><span class="api-parameter">match_whole_word</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation">?</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeRange.html">CodeRange</a><span class="api-punctuation">?</span></code> {#code-find-text-match_case-match_whole_word}

<div class="api-description">

Finds the first literal occurrence in the code at the current scene time. Match_case and match_whole_word default to false. Whole words use letters, digits, and underscores; adjacent punctuation is allowed. Returns a CodeRange to pass to select, highlight, replace, or erase, or nil when absent. The result is a fixed range, not a live query.

```lua
local range = code:find('value')
if range then
code:replace(range, 'answer'):play(1)
end
```

</div>

</section>

<section class="api-method">

## <code class="api-signature"><a class="api-type-name" href="Code.html">Code</a><span class="api-punctuation">:</span><span class="api-function">find_all</span><span class="api-punctuation">(</span><span class="api-parameter">text</span><span class="api-punctuation">: </span><span class="api-type-name">string</span><span class="api-punctuation">, </span><span class="api-parameter">match_case</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation">?</span><span class="api-punctuation">, </span><span class="api-parameter">match_whole_word</span><span class="api-punctuation">: </span><span class="api-type-name">boolean</span><span class="api-punctuation">?</span><span class="api-punctuation">)</span><span class="api-punctuation">: </span><a class="api-type-name" href="CodeRange.html">CodeRange</a><span class="api-punctuation">[</span><span class="api-punctuation">]</span></code> {#code-find_all-text-match_case-match_whole_word}

<div class="api-description">

Finds all non-overlapping literal occurrences at the current scene time. Match_case and match_whole_word default to false. Whole words use letters, digits, and underscores; adjacent punctuation is allowed. Returns an empty list when absent. Returned coordinates do not follow later edits.

```lua
code:highlight(code:find_all('value')):play(1)
```

</div>

</section>

</div>

:::
