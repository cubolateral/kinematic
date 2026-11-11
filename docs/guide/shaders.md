---
pageClass: guide-page
---

# Shaders

A shader computes colors on the GPU. Use an image shader to process a captured object
or draw a procedural image on a 2D plane. Use a mesh shader to shade 3D geometry using
its positions and normals. Lua supplies uniforms that you can animate like properties.

## Draw a Mandelbrot set on a plane

Each pixel represents a complex number `c`. Starting at zero, repeatedly compute
`z = z² + c`. Points that escape the radius-2 boundary receive a color based on their
iteration count. Points that remain inside after 200 steps are black.

<GuideDemo name="shader-mandelbrot" caption="A full-frame Mandelbrot view magnifies its boundary 128 times in black and blue.">

<!-- focus: shader-mandelbrot -->
```lua
local plane = k.world_2d():add(k.panel{
    size = k.vec2(960, 540),
    shader = fractal,
    uniforms = { zoom = 1 },
})

plane:uniform("zoom"):to(128):play(2, "quart_in_out")
plane:uniform("zoom"):to(1):play(2, "quart_in_out")
```

<template #source>

<!-- demo: shader-mandelbrot -->
<<< ./projects/shaders/scenes/01-shader-mandelbrot.lua

</template>

</GuideDemo>

The rectangle matches the guide resolution, 960 × 540. Adjust its size to your project
resolution to fill another frame. Its white fill supplies alpha coverage. The shader replaces its
RGB with the fractal. Expand **Complete scene** for the full program. Its central
calculation is:

```glsl
vec2 c = vec2(-0.7435, 0.1314) + (uv - 0.5) * vec2(3.5556, 2.0) / zoom;
vec2 z = vec2(0.0);
int steps = 0;
for (int i = 0; i < 200; i++) {
    z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
    if (dot(z, z) > 4.0) break;
    steps++;
}
```

`uv` covers the captured image from zero to one. Dividing the offset by `zoom` shows a
smaller region of the complex plane, magnifying its detail without resizing the object.
The view zooms up to 128× around a detailed boundary point. Quartic easing makes
the wide zoom range easier to follow. The fixed iteration limit bounds work per pixel.
Very deep zooms need more iterations and eventually more numeric precision.

## Bind and animate uniforms

The shader declares `uniform float zoom`, and the object supplies `uniforms = {zoom = 1}`.
Their types must agree. `plane:uniform("zoom")` returns the binding's property handle.
Animating it updates the GPU value without recreating the shader.

Use `to`, `by`, or `from` as for ordinary properties. To change several bindings at
once, use `plane:to {uniforms = {zoom = 2}}:play(2)`. Uniforms have their own namespace
and do not become built-in object fields.

## Image coordinates and transparency

`k.image_shader` takes a complete GLSL 3.30 fragment shader, including `#version 330 core`.
You can keep it in a file with `k.image_shader {path = "assets/fractal.glsl"}`.
Paths are relative to the project root.

| Input | Meaning |
| --- | --- |
| `k_image` | The object's captured image. |
| `k_resolution` | Capture dimensions in pixels. |
| `k_time` | Evaluated scene time in seconds. |
| `k_alpha` | Object opacity, already included in the captured image. |

UV `(0, 0)` is the bottom-left corner. Captures use premultiplied-alpha sRGB RGBA.
The example samples alpha from `k_image` and multiplies the generated RGB by it.
Do not multiply captured colors by `k_alpha` again.

Blur or glow that samples neighboring pixels may need extra `shader_padding` to avoid
cut edges. Larger captures cost more to render.

## Animate a surface pattern in 3D

This sphere has flowing bands calculated from world position. A nested sine bends
them around the surface, while an animated phase moves the pattern through one full
cycle. A simple normal-based light calculation reveals the sphere's shape.

<GuideDemo name="shader-surface" caption="Blue and white bands flow over a shaded sphere in a continuous loop.">

<!-- focus: shader-surface -->
```lua
local sphere = k.world_3d():add(k.sphere{
    radius = 0.8,
    segments = 64,
    shader = material,
    uniforms = { phase = 0 },
})

sphere:uniform("phase"):to(math.pi * 2):play(2, "linear")
```

<template #source>

<!-- demo: shader-surface -->
<<< ./projects/shaders/scenes/02-shader-surface.lua

</template>

</GuideDemo>

The pattern uses `k_world_position`, and its light uses `k_normal`. Expand the scene
for the complete GLSL. These lines control the pattern and its edge softness:

```glsl
float wave = sin(p.y * 16.0 + 3.0 * sin(p.x * 5.0 + phase));
float band = smoothstep(-0.2, 0.2, wave);
```

Increasing `16.0` makes more bands. Increasing `3.0` bends them more strongly. The
sphere stays still, so every visible change comes from the shader. Phase advances by
one full turn with linear easing and no final pause, making the video repeat smoothly.

Mesh sources omit `#version` because the engine assembles the program. The default
vertex stage supplies `k_world_position`, `k_normal`, and `k_uv`. Custom mesh shaders
replace built-in lighting. Scene lights do not affect this example's explicit light
calculation.

## Additional textures and vertices

An image shader can sample another canvas through a named sampler and an object binding
such as `textures = {other = canvas:texture()}`. Dependencies must stay within the scene
and cannot form cycles. The current mesh path does not bind samplers through `textures`.

A custom vertex stage uses `vertex` or `vertex_path`. It must actively use `modelMatrix`
and `viewProjection`, and supply the outputs its fragment shader reads. GPU deformation
does not update CPU bounds. See [Shader](../api/Shader.md) for the full contract.

## Check the result

`--check` validates scene construction and bindings but does not compile GLSL. Render
the scene to test the program, including transparent edges and the extremes of animated
uniforms. Shader source and resources stay fixed within an applied revision.

A failed shader edit preserves the last valid preview. Read the diagnostics if the
image does not change after an edit. Use uniforms for animation rather than reloading
source every frame.
