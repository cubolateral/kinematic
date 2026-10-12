# Lua implementation inventory

The Rust source remains authoritative. This inventory was taken before adding
bindings, from `src/lib.rs`, `src/prelude.rs`, and their public modules.

| Family | Existing API to integrate |
| --- | --- |
| Application | App, scene factories, SceneBuilder, project settings, export. |
| Values | Vector2, Vector3, Quaternion, Color, Quad, geometry Rect, Random, Easing. |
| 2D objects | Circle, Ellipse, Rect, RegularPolygon2D, Line2D, Image2D, Text2D, Latex2D, Group2D, Container2D, Canvas2D, Projection2D. |
| 3D objects | Sphere, Torus, Prism, Pyramid, Plane, RegularPolygon3D, Line3D, Image3D, Text3D, Latex3D, Light3D, Group3D, Container3D, Canvas3D, Projection3D. |
| Components | Transform2D/3D, Style, Filter, Draw2D/3D, Camera2D/3D, Material, View, Inspection, Simulation. |
| Properties | Macro-generated TrackableInfo/TrackInfo, TrackHandle, subchannels, shader uniforms, standalone tracks. |
| Scheduling | Task, Tween, play, wait, delay, chain, all, parallel, repeat, named temporal events. |
| Hierarchy | Root, Node, attach, ordering, parent/ancestor, local/global transforms, anchors, logical/visual bounds, lifetime removal. |
| State | Snapshot, save/restore/reset, temporal signals, fixed-step simulation, manual updates, scheduled writes and checkpoints. |
| Resources | Images, fonts, LaTeX, CanvasTexture, ProjectionSource, ImageShader, MeshShader, uniforms, mesh geometry, materials and lights. |
| Extension | Object/Node/Trackable/TrackEnum derives, native object registration, custom 2D paths and 3D meshes. |

Native property names, types, limits and choices must come from generated
metadata, rather than a second handwritten catalogue. External hecs, Skia,
glam and three-d reexports do not imply full Lua bindings for those libraries.

Built-in effects and particle transitions were removed by the maintainer.
Animate opacity, position, scale and other tracks explicitly in Rust or Lua.
