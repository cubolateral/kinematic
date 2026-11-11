---
pageClass: guide-page
---

# 3D graphics

## Build a lit scene

Start with a world, a primitive, and a light. Use a slow rotation to reveal how the
material responds across the surface.

<GuideDemo name="lit-cube" caption="A complete turn reveals the cube faces under the default world lighting.">

<!-- demo: lit-cube -->
<<< ./projects/graphics-3d/scenes/01-lit-cube.lua

</GuideDemo>

`unlit = true` makes color independent of lights. Try it first, then return to lit
rendering and compare roughness `0.1` with `0.9`. Metallic changes how the surface
reflects light. Roughness changes how concentrated those reflections are. Keep framing
and lighting fixed when comparing materials. Point lights vary with position, while
spot lights also restrict their cone. The options for each light are type-specific.

## Primitives and presets

The scene provides `sphere`, `torus`, `prism`, `pyramid`, `plane`, `regular_polygon_3d`,
and `line_3d`. Presets such as `triangle_3d`, `square_3d`, `circle_3d`, `tetrahedron`,
`cube`, `cylinder`, and `cone` use built-in geometry with preset parameters. Compare
their option types under [k](../api/k.md). Preset defaults are generated from
the runtime too.

Attach primitives to `k.world_3d()` and select `k.view("3d")`, or use a separate 3D
canvas. Position and scale use `vec3`, and orientation uses a quaternion.

## Materials and lights

Built-in materials use `albedo`, `metallic`, `roughness`, `opacity`, `unlit`,
`outline_color`, and `outline_width`. Set `unlit=true` to ignore lighting, which is
useful for diagrams with uniform colors. Lit materials limit roughness to 0.04–1 and
metallic to 0–1.

The scene provides directional (`directional_light`), point (`point_light`), and spot
(`spot_light`) lights, each with its own properties. Attach lights to the same world or
canvas as the objects they illuminate. A custom mesh shader replaces the default
material lighting.

The [3D project](../examples/3d.md) demonstrates a material and lighting. See
[Sphere](../api/Sphere.md), [DirectionalLight3D](../api/DirectionalLight3D.md),
[PointLight3D](../api/PointLight3D.md), and [SpotLight3D](../api/SpotLight3D.md) for
their options.

## Choose a light

Ambient light sets the canvas's uniform illumination. Directional, point, and spot
lights use different spatial models. The examples below use a pyramid, a cube,
and a sphere to show each effect. Light objects must be attached to the
canvas containing the objects they illuminate.

### Ambient: uniform canvas lighting

Each 3D canvas owns `ambient_color` and `ambient_intensity`. The defaults are white
and `0.4`. Ambient light illuminates every surface uniformly, independent of position
or orientation. Intensity must be nonnegative; color alpha does not affect lighting.

<GuideDemo name="light-ambient" caption="Three white shapes rotate as ambient light dims, brightens, then shifts through red, green, and blue.">

<!-- focus: light-ambient -->
```lua
k.world_3d().ambient_intensity:to(0):play(1)
k.wait(0.5)
k.world_3d().ambient_intensity:to(1):play(1)

k.world_3d().ambient_color:to(k.color("#FF0000")):play(1)
k.world_3d().ambient_color:to(k.color("#00FF00")):play(1)
k.world_3d().ambient_color:to(k.color("#0000FF")):play(1)
```

<template #source>

<!-- demo: light-ambient -->
<<< ./projects/graphics-3d/scenes/02-light-ambient.lua

</template>

</GuideDemo>

The example keeps the automatic directional light, which remains active even when
ambient intensity is zero. Add a directional light with `intensity = 0` to disable
that fallback and isolate ambient lighting. These properties also apply to separate `canvas_3d`
objects. See [Canvas3D](../api/Canvas3D.md) and [Canvas3DOptions](../api/Canvas3DOptions.md).

### Directional: one direction everywhere

A directional light has parallel rays along its local negative Z axis. Position does
not affect its illumination. Rotating the light changes which surfaces receive light. In this example the light stays
fixed while the group of shapes rotates.

<GuideDemo name="light-directional" caption="Three colored shapes rotate together under a fixed directional light.">

<!-- focus: light-directional -->
```lua
local light = k.directional_light{
    rotation = k.axis_angle(k.vec3(0, 1, 0), -1),
}
k.world_3d():add(light)
```

<template #source>

<!-- demo: light-directional -->
<<< ./projects/graphics-3d/scenes/03-light-directional.lua

</template>

</GuideDemo>

Use this for distant, consistent illumination, such as sunlight across a diagram.
Changing `intensity` adjusts its strength without altering direction.

### Point: a source at a position

A point light shines in all directions from its position. Here it moves across the
shapes. The small white marker shows the light's location, while distance falloff
makes nearby surfaces brighter.

<GuideDemo name="light-point" caption="A moving point light brightens the surfaces nearest to its white position marker.">

<!-- focus: light-point -->
```lua
local light = k.point_light{
    position = k.vec3(-1.75, 0.75, 1.75),
    intensity = 5,
    quadratic = 0.25,
}
```

<template #source>

<!-- demo: light-point -->
<<< ./projects/graphics-3d/scenes/04-light-point.lua

</template>

</GuideDemo>

Falloff is `1 / (constant + linear × distance + quadratic × distance²)`. The defaults
are `constant = 1`, `linear = 0`, and `quadratic = 0`, so distance falloff must be
configured explicitly. This example sets `quadratic = 0.25`.

### Spot: a directed cone

A spotlight combines a position with a cone along its local negative Z axis. Rotate
it to aim the cone and use `cutoff`, in radians, to control its half-angle. The narrow
cone below sweeps across the row.

<GuideDemo name="light-spot" caption="The spotlight cone sweeps across the three colored shapes.">

<!-- focus: light-spot -->
```lua
local light = k.spot_light{
    position = k.vec3(0, 0, 2),
    rotation = k.axis_angle(k.vec3(0, 1, 0), -1),
    intensity = 5,
    quadratic = 0.1,
    cutoff = 0.4,
}
k.world_3d():add(light)

light.rotation:to(k.axis_angle(k.vec3(0, 1, 0), 1)):play(2)
light.rotation:to(k.axis_angle(k.vec3(0, 1, 0), -1)):play(2)
```

<template #source>

<!-- demo: light-spot -->
<<< ./projects/graphics-3d/scenes/05-light-spot.lua

</template>

</GuideDemo>

Increase `cutoff` to illuminate a wider area. Like point lights, spotlights have
configurable distance falloff. Both position and orientation matter when aiming them.
A custom mesh shader replaces the built-in lighting calculation.

See [DirectionalLight3D](../api/DirectionalLight3D.md),
[PointLight3D](../api/PointLight3D.md), and [SpotLight3D](../api/SpotLight3D.md) for options.


## Compare materials under the same light

A material comparison is easier when geometry, camera, and light stay fixed.

<GuideDemo name="material-compare" caption="Left: default dielectric. Middle: rough metal. Right: unlit white." still>

<!-- demo: material-compare -->
<<< ./projects/graphics-3d/scenes/06-material-compare.lua

</GuideDemo>

The right sphere ignores the light and shows a flat color. Change only roughness on the
left sphere for a direct comparison of highlight spread. A metallic surface also needs
suitable illumination to remain readable. Increasing albedo is not a substitute for
framing a light.

## Cameras

Every 3D canvas has a camera. Its position, orientation, projection mode, field of view,
and clipping planes determine the view. The [camera guide](cameras.md) explains each
control and `focus_camera`, with separate 2D and 3D examples.

## Quaternions

`k.quat(x,y,z,w)` creates a quaternion value. `k.axis_angle(axis,radians)` creates an
orientation from an axis and angle. Multiplication combines orientations and can rotate
vectors. Interpolating orientations does not represent complete turns. To preserve full
turns, use the quaternion handle's rotation method:

```lua
cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(2, "linear")
```

See [Quaternion](../api/Quaternion.md) and
[QuaternionProperty](../api/QuaternionProperty.md).

## Text, LaTeX, and images

Use `text_3d`, `latex_3d`, and `image_3d` to place content in 3D space. `text_3d`
accepts project fonts, while `latex_3d` uses the built-in math renderer. Each type has
its own properties and transitions. Do not assume every Text2D method is available on
Text3D. The API reference lists them separately.

Images come from `k.load_image(path)`, while projections use `canvas:texture()`. A 2D canvas
can feed a 3D projection to create a panel in space. A 3D canvas can feed a 2D
projection to compose 3D content into a 2D world.

## Compose across dimensions

In the [composition project](../examples/composition.md), the root contains the source
canvas and the 2D world contains a projection that displays it. See [canvases and
projections](canvases.md) for resolution, dependencies, and lifetime. Texture resolution
determines its pixels, while projection `size` determines its area in the composition.
Avoid texture cycles and do not share a texture across scenes.

Continue with [custom drawing and shaders](drawing.md).
