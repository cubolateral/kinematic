# Shaders

Shaders can process a captured image or replace a mesh material program. These paths
have different contracts. Choose based on the input your program needs, not only the
dimensions of the final composition.

## Textures and uniforms

Load images with `k.image(path)` and fonts with `k.font(path)`. Canvases provide
textures through `texture()`. Resources and their bytes are immutable within an applied
revision. Canvas dependencies cannot form cycles or cross scenes.

Declare uniforms and textures in the options before animating them, for example
`uniforms={pulse=0}` and `textures={other=canvas:texture()}`. `object:uniform("pulse")`
returns a handle. `object:to {uniforms={pulse=6}}:play(3)` animates it without
conflicting with regular properties. Uniforms support booleans, numbers, vectors,
quaternions, quads, and colors. Strings and enums have no GLSL representation.

## Image shaders

`k.image_shader {source=glsl}` or `{path="assets/image.glsl"}` accepts a complete GLSL
3.30 fragment shader, including `#version`. If both fields are set, the file takes
priority. Set `shader` on the object and use `shader_padding` to expand the capture for
blur or glow.

The engine provides `k_image`, `k_resolution` in pixels, `k_time` in scene seconds, and
`k_alpha`. UV `(0,0)` is the bottom-left pixel. Input and output use premultiplied-alpha
sRGB RGBA. Preserve this format when changing colors.

<<< ../../examples/lua/shaders/scenes/intro.lua

## Mesh shaders

`k.mesh_shader {fragment=glsl,vertex=optional_glsl}` also accepts `fragment_path` and
`vertex_path`. File paths take priority over source strings. Omit `#version` from mesh
sources. The default vertex stage provides `k_world_position`, `k_normal`, and `k_uv`.

A custom vertex stage uses the `position` attribute and may use `normal` and
`uv_coordinates`. It must actively use `modelMatrix` and `viewProjection`. Optional
engine uniforms include `normalMatrix`, `viewMatrix`, `projectionMatrix`,
`cameraPosition`, `sceneTime`, linear `materialColor`, `hasNormal`, and `hasUv`.

Custom shaders replace built-in lighting. Children inherit group shaders and uniforms. A
local shader takes priority, and compatible local uniforms replace inherited values. GPU
deformation does not update CPU geometry bounds. Adjust padding when needed for
selection and culling.

The [shader project](../examples/shaders.md) demonstrates both contracts. `--check` does
not compile GLSL. Run `--render` and inspect the frames. Compilation errors leave the
last valid preview in place.

## Set a shader on an instance

Create the resource once and provide its bindings in the object options. File paths are
relative to the project root. Resource contents are frozen into the applied revision.

```lua
local shader = k.mesh_shader {
    fragment_path = "assets/material.glsl",
}

local cube = s:cube {
    shader = shader,
    uniforms = {pulse = 0},
    textures = {},
}
s:world_3d():add(cube)

cube:uniform("pulse"):to(6):play(2)
```

The GLSL declaration for `pulse` must match the binding type. An unknown or incompatible
uniform does not become an object property. Regular properties and uniforms use separate
namespaces: `object:prop("position")` accesses a built-in property, while
`object:uniform("pulse")` accesses a shader binding.

## Values, colors, and interpolation

Numbers, booleans, and supported composite values are converted to matching uniforms.
Mesh material colors use linear space. Captured images use the premultiplied sRGB format
described above. Mixing these formats can produce incorrect brightness or edges.

Animate declared values with `to`, `by`, or `from`, using `uniforms={...}` in the
destination. Preparing an animation does not replace the shader resource. Canvas
textures are bindings for image shaders. The current mesh path does not configure
samplers through `textures`, so do not assume the same binding works for both contracts.
Program sources and textures belong to the revision. They are not channels that change
each frame.

## Custom vertex shaders

A custom vertex shader can deform geometry or change its transform, but must preserve
the attribute and matrix contract. It must actively use `modelMatrix` and
`viewProjection`. Declaring a uniform that the compiler removes does not meet this
requirement.

If the fragment shader reads `k_world_position`, `k_normal`, or `k_uv`, the vertex
shader must provide matching outputs. Do not include `#version` in mesh sources. The
engine assembles the version and final program.

CPU geometry bounds do not follow GPU deformation. Set bounds and padding so selection,
framing, and culling account for the visible result. Changing uniforms should not
rebuild the mesh.

## Inheritance, capture, and render cost

Children can inherit a compatible shader set on a group. A child shader takes priority,
and compatible local uniforms replace inherited ones. This does not unify the image and
mesh contracts.

`shader_padding` expands the capture for effects that need pixels beyond the object. Use
only the space needed for blur or glow, since large captures cost more to render. An
effect that reads another canvas texture adds a render dependency, which must not create
a cycle.

## Validate and troubleshoot

1. Run `--check` to find construction and binding problems.
2. Run `--render` with OpenGL 3.3 available to compile and draw the program.
3. Check frames at the start, middle, and end. Animated uniforms can reveal issues a poster does not show.
4. Check transparency and edges against the final composition background.

Compilation errors preserve the previous preview revision. A visible preview does not
prove that new source was applied. Read the status and diagnostics. During export,
resources remain fixed to the applied revision.

The [shader project](../examples/shaders.md) compares image and mesh shaders. The
[custom object project](../examples/custom-object.md) shows how to provide custom
geometry before adding GPU programs. See [Shader](../api/Shader.md),
[ResourceOptions](../api/ResourceOptions.md), and [k](../api/k.md).
