# Guide projects

Each folder is an editable Kinematic project for one guide page. Open the folder in
Kinematic, or run `cargo run -- docs/guide/projects/text` from the repository root.

Scenes are numbered in explanation order and listed in that order in
`kinematic/project.ron`. Each scene produces one page demo. Static scenes intentionally
have no duration. The guide includes the complete scene directly from its Lua file.

Edit these files directly. Documentation commands do not recreate or overwrite them.
Keep scene names and page includes aligned when adding or moving a demonstration.
Update a focused excerpt in the Markdown if its demonstrated operation changes.

Run `npm --prefix docs run docs:examples -- --guide-only` to validate the projects
without exporting. Rendering is a separate, explicit command. Generated editor metadata,
caches, and exports are ignored by Git.

| Guide page | Project | Scenes |
| --- | --- | --- |
| [Animation](../animation.md) | [animation](animation/kinematic/project.ron) | 2 |
| [Audio](../audio.md) | [audio](audio/kinematic/project.ron) | 1 |
| [Cameras](../cameras.md) | [cameras](cameras/kinematic/project.ron) | 3 |
| [Canvases and projections](../canvases.md) | [canvases](canvases/kinematic/project.ron) | 2 |
| [Code](../code.md) | [code](code/kinematic/project.ron) | 11 |
| [Coordinates, transforms, and anchors](../coordinates.md) | [coordinates](coordinates/kinematic/project.ron) | 2 |
| [Custom objects](../custom-objects.md) | [custom-objects](custom-objects/kinematic/project.ron) | 1 |
| [Procedural drawing](../drawing.md) | [drawing](drawing/kinematic/project.ron) | 2 |
| [Easing](../easing.md) | [easing](easing/kinematic/project.ron) | 2 |
| [2D graphics](../graphics-2d.md) | [graphics-2d](graphics-2d/kinematic/project.ron) | 1 |
| [3D graphics](../graphics-3d.md) | [graphics-3d](graphics-3d/kinematic/project.ron) | 5 |
| [Images](../images.md) | [images](images/kinematic/project.ron) | 5 |
| [LaTeX](../latex.md) | [latex](latex/kinematic/project.ron) | 5 |
| [Groups and layout](../layout.md) | [layout](layout/kinematic/project.ron) | 2 |
| [Lua conventions](../lua.md) | [lua](lua/kinematic/project.ron) | 2 |
| [Objects and composition](../objects.md) | [objects](objects/kinematic/project.ron) | 2 |
| [Random](../random.md) | [random](random/kinematic/project.ron) | 2 |
| [Sequences, parallel work, and events](../sequences.md) | [sequences](sequences/kinematic/project.ron) | 2 |
| [Shaders](../shaders.md) | [shaders](shaders/kinematic/project.ron) | 2 |
| [Signals](../signals.md) | [signals](signals/kinematic/project.ron) | 2 |
| [Simulation](../simulation.md) | [simulation](simulation/kinematic/project.ron) | 2 |
| [Get started](../start.md) | [start](start/kinematic/project.ron) | 4 |
| [Text](../text.md) | [text](text/kinematic/project.ron) | 6 |
