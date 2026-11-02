# 3D graphics

## Primitives and presets

The scene provides `sphere`, `torus`, `prism`, `pyramid`, `plane`, `regular_polygon_3d`,
and `line_3d`. Presets such as `triangle_3d`, `square_3d`, `circle_3d`, `tetrahedron`,
`cube`, `cylinder`, and `cone` use built-in geometry with preset parameters. Compare
their option types under [Scene](../api/Scene.md). Preset defaults are generated from
the runtime too.

Attach primitives to `s:world_3d()` and select `s:view("3d")`, or use a separate 3D
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
cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, k.ease.linear)
```

See [Quaternion](../api/Quaternion.md) and
[QuaternionProperty](../api/QuaternionProperty.md).

## Text, LaTeX, and images

Use `text_3d`, `latex_3d`, and `image_3d` to place content in 3D space. `text_3d`
accepts project fonts, while `latex_3d` uses the built-in math renderer. Each type has
its own properties and transitions. Do not assume every Text2D method is available on
Text3D. The API reference lists them separately.

Images come from `k.image(path)`, while projections use `canvas:texture()`. A 2D canvas
can feed a 3D projection to create a panel in space. A 3D canvas can feed a 2D
projection to compose 3D content into a 2D world.

## Compose across dimensions

In the [composition project](../examples/composition.md), the root contains the source
canvas and the 2D world contains a projection that displays it. See [canvases and
projections](canvases.md) for resolution, dependencies, and lifetime. Texture resolution
determines its pixels, while projection `size` determines its area in the composition.
Avoid texture cycles and do not share a texture across scenes.

<<< ../../examples/lua/composition/scenes/intro.lua

Continue with [custom drawing and shaders](drawing.md).
