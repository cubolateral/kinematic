# Lua implementation inventory

Projects and scenes are written in Lua. Rust source remains authoritative for
the internal runtime and the object metadata exposed to Lua.

| Family | Internal runtime family |
| --- | --- |
| Application | Executable, Lua project loading, project settings, export. |
| Values | Vector2, Vector3, Quaternion, Color, Quad, geometry Rect, Random, Easing. |
| 2D objects | Circle, Ellipse, Rect, RegularPolygon2D, Line2D, Image2D, Text2D, Latex2D, Group2D, Container2D, Canvas2D, Projection2D. |
| 3D objects | Sphere, Torus, Prism, Pyramid, Plane, RegularPolygon3D, Line3D, Image3D, Text3D, Latex3D, Light3D, Group3D, Container3D, Canvas3D, Projection3D. |
| Components | Transform2D/3D, Style, Filter, Draw2D/3D, Camera2D/3D, Material, View, Inspection, Simulation. |
| Properties | Macro-generated TrackableInfo/TrackInfo, TrackHandle, subchannels, shader uniforms, standalone tracks. |
| Scheduling | Task, Tween, play, wait, delay, chain, all, parallel, repeat, named temporal events. |
| Hierarchy | Root, Node, attach, ordering, parent/ancestor, local/global transforms, anchors, logical/visual bounds, lifetime removal. |
| State | Snapshot, save/restore/reset, temporal signals, fixed-step simulation, manual updates, scheduled writes and checkpoints. |
| Resources | Images, fonts, LaTeX, CanvasTexture, ProjectionSource, ImageShader, MeshShader, uniforms, mesh geometry, materials and lights. |
| Extension | Lua components and objects, custom 2D paths and 3D meshes. |

Native property names, types, limits and choices must come from generated
metadata, rather than a second handwritten catalogue. The internal hecs, Skia, glam and three-d dependencies have no public Rust API.

Built-in effects and particle transitions were removed by the maintainer.
Animate opacity, position, scale and other tracks explicitly in Lua.
