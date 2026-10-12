# Lua projects

Build the editor with `cargo build --bin kinematic`. Lua 5.4 is embedded; no Lua
installation is needed. Open the start screen with `cargo run --bin kinematic`,
or select a project directly:

```sh
cargo run --bin kinematic -- examples/lua/2d
cargo run --bin kinematic -- --new /tmp/my-animation "My animation"
cargo run --bin kinematic -- --new /tmp/my-3d-animation "My 3D animation" --3d
cargo run --bin kinematic -- --check examples/lua/simulation
cargo run --bin kinematic -- --render examples/lua/3d
```

Rendering needs an OpenGL 3.3 desktop session and FFmpeg on `PATH`. `--check`
validates construction and evaluation without opening a window; it does not
validate GLSL compilation or GPU drawing. `--render` uses the application's
existing SDL/OpenGL/Skia/three-d resources and writes `output/<project name>.mp4`.
Existing exports are preserved by adding a numeric suffix to the new filename.

## Files and editing

`kinematic.ron` is the authoritative configuration:

```ron
(
    version: 1,
    name: "My animation",
    scenes: ["scenes/intro.lua"],
    resolution: (1280, 720),
    fps: 60,
    margin: (0, 0),
)
```

Paths are relative to this directory, including module imports, fonts, images,
shader files, caches and output. Absolute paths, parent traversal and symlinks
that leave the project are rejected. `require("folder.module")` resolves
`folder/module.lua` or `folder/module/init.lua`. Imports are cached per revision;
cyclic imports are rejected.

The Project panel creates, opens and closes projects without restarting the
application. Files supports opening, creating and renaming files and folders.
Renaming does not rewrite imports. Code tabs use Dear ImGui CTE with independent
selection, undo/redo, find/replace, Lua highlighting and a monospaced font.
Closing unsaved documents or a project, or exiting, offers Save, Discard and
Cancel. Saves use a temporary file and atomic rename, with a disk-version check.
An external edit conflicting with an unsaved buffer offers explicit resolution.

After 400 ms without editing, the editor builds from the buffers without saving.
The status reports pending work, replay or failure. Diagnostics open the relevant
file, line and column; Console retains at most 500 lines and suppresses
consecutive identical prints. Runtime failures keep
the previous preview visible. Expensive simulation replay proceeds in short
slices and can be cancelled. Spaces, arrows, B, F and Esc in text inputs do not
control playback, rebuild, fullscreen or application exit.

LuaLS is optional for execution. Install it yourself, set `KINEMATIC_LUALS` to
its executable, use an existing Neovim Mason installation, or put
`lua-language-server` on `PATH` (in that search order). Nothing is downloaded.
Each project owns an asynchronous server, with generated API definitions and
logs in `kinematic/cache/luals`. User LuaLS settings are not overwritten.
Completion is available with Ctrl+Space, documentation with F1, definition with
F12, and signatures through the Signature button. Buffers and version numbers
are sent to the server; stale replies are discarded. UTF-8/UTF-16 positions are
converted to the editor's character coordinates. Server failure is a diagnostic
and does not disable basic editing or execution.

## Scene API

```lua
local k = require("kinematic")
return k.scene("Intro", function(s)
    local ball = s:circle {
        id = "ball", radius = 24,
        position = k.vec2(-100, 0), fill = k.color("#60A5FA"),
    }
    s:world_2d():add(ball)
    ball.opacity:set(0)
    s:all(function()
        s:play(ball.position:to(k.vec2(100, 0), 2, k.ease.linear))
        s:play(ball.opacity:to(1, 0.5))
    end)
    s:wait(1)
end)
```

Native constructor names and directly exposed properties come from the Rust
`Object` and `Trackable` macros. The generated file
`kinematic/cache/luals/kinematic.lua` lists their types, and
[the inventory](lua-parity.md) lists the API families. Constructors include
`circle`, `ellipse`, `rect`, `regular_polygon_2d`, `line_2d`, `image_2d`,
`text_2d`, `latex_2d`, `group_2d`, `container_2d`, `canvas_2d`, `projection_2d`,
`sphere`, `torus`, `prism`, `pyramid`, `plane`, `regular_polygon_3d`, `line_3d`,
`image_3d`, `text_3d`, `latex_3d`, `group_3d`, `container_3d`, `canvas_3d`,
`projection_3d`, `directional_light`, `point_light` and `spot_light`.
Polygon presets include `triangle`, `square`, `triangle_3d`, `square_3d`,
`circle_3d`, `tetrahedron`, `cube`, `cylinder` and `cone`.

Objects start inactive. Attach them using `s:world_2d():add(obj)`,
`s:world_3d():add(obj)` or a group's `add`. Extra canvases attach to `s:root()`.
`root.view_2d:set(false)` selects the 3D view. Native dimensions, transforms,
opacity, material, camera, filter, light and layout properties use the same
handles as other tracks. Enum values use the Rust variant names.

| Operation | Meaning |
| --- | --- |
| `property:get()` | Read the current value. |
| `property:set(value)` | Set without scheduling an animation. |
| `property:to(value, seconds, easing)` | Prepare an absolute tween. |
| `property:by(delta, seconds, easing)` | Prepare a relative number/vector tween. |
| `property:rotate(axis, radians, seconds, easing)` | Quaternion rotation preserving full turns. |
| `property:from(start, finish, seconds, easing)` | Prepare a tween with an explicit start. |
| `property.x`, `.y`, `.z`, `.w` | Access a subchannel; colors also accept `.r`, `.g`, `.b`, `.a`. |
| `task:delay(seconds)` | Prepare a delayed task. |
| `s:play(task)` | Compile the task into the current scheduling scope. |
| `s:wait(seconds)` | Advance that scope. |
| `s:event(name)` | Named wait editable in Timeline. |
| `s:chain(fn)` | Run tasks sequentially. |
| `s:all(fn)` | Start tasks together and advance by the longest duration. |
| `s:parallel(fn)` | Start a parallel branch without advancing the parent scope. |
| `s:loop(fn)` | Repeat a positive finite cycle without advancing the parent scope. |
| `s:track(initial, id)` | Independent animated value, usable as a simulation input. |

Tweens default to one second and the native easing default. During construction,
preparing a tween advances its property's builder value to the target, as in
Rust, so later tasks and `get()` can use it; only `play` schedules evaluation.
`k.ease` uses lowercase snake case names such as `linear`, `cubic_out` and
`sine_in_out`. Nested repeats and ambiguous concurrent writes within repeats are
rejected by the native scheduler. Keyframes remain in timeline order.

Hierarchy queries include `children`, `get_child(index)`, `parent`,
`ancestor(type_name)` and `type_name`. Child positions are one-based.
`insert(child,index)`, `move_to`, `move_up`, `move_down`, `move_to_top` and
`move_to_bottom` preserve native ordering. `remove` and `remove_children` end
lifetimes; seeking backwards can restore the objects.

`bounds` returns the local box size; `middle`, `left`, `right`, `top`, `bottom`
and corners return points in parent coordinates, including displaced bounds.
3D adds `front`, `back` and their edge/corner combinations, for example
`top_left_front`. Each anchor has `*_to(point)` returning the required local
position. Global queries are `global_position`, `global_rotation`,
`global_scale`, `global_matrix`, and 2D `global_opacity`. Image objects support
`get_pixel_color(vec2)`. A 3D canvas supports
`focus_camera(position,target,seconds,easing)`, returning a task.

`snapshot()` captures tracked fields, dynamic properties and uniforms.
`restore(snapshot,seconds,easing)` creates a tween to that snapshot.
`save()` pushes a snapshot, `restore(nil,seconds,easing)` pops it, and
`reset(seconds,easing)` returns to builder values. These return tasks for `play`.

Built-in effects have been removed from both Rust and Lua, including fade, grow,
creation, write, morph and text/LaTeX transitions. Animate opacity, scale,
position, text and other properties manually.

## Signals and simulations

`obj:signal(function(obj, frame) ... end)` and
`property:signal(function(property, frame) ... end)` return a handle with
`stop()`, scheduled at the current timeline position. `frame` contains `time`,
`index` and `dt`. Signals run after tracks while their object is active; their
writes are temporary and restored before the next evaluation. Failed callbacks
do not commit their staged writes. Signals cannot add objects or schedule work.

```lua
local component = k.component {speed = {default = 20, min = 0, max = 100}}
local moving = k.object {
    dimension = 2,
    components = {component},
    state = {distance = 0, random = k.random(42)},
    inputs = {speed = "speed"},
    update = function(state, frame, inputs)
        state.distance = state.distance + inputs.speed * frame.dt
    end,
    bounds = function() return {0, -10, 200, 20} end,
    draw = function(ctx, properties, state)
        ctx:circle(state.distance, 0, 10, properties.fill)
    end,
}
```

Instantiate with `s:object(moving, {id="moving", speed=40})`. Dynamic fields are
ordinary property handles, shown in Inspector and Timeline. Defaults determine
the type; optional bounds and string choices are validated. Component properties
must not shadow native properties. Each instance receives independent state.

Declare inputs as `{speed="speed"}` for an own property,
`{speed=other.speed}` for a property handle, or `{source=other}` for another
simulation's copied state. `obj:inputs({...})` sets them after construction.
If `inputs` is omitted, own properties are conservatively treated as inputs.
Dependencies are sampled at the fixed step's time, not the final seek time.
Simulation cycles are rejected; dependencies run before dependent steps.
`auto_update:set(false)` disables automatic updates. `update()`, `checkpoint()`
and `mutate(function(state) ... end)` schedule explicit operations.
`state()` returns an independent copy for inspection.

Update and mutation callbacks cannot capture upvalues other than the restricted
read-only environment, isolated from construction-time library mutations. Inside a callback, `local k = require("kinematic")`
provides value, Random, path, matrix and mesh constructors without capturing the
outer module; other module imports are restricted to construction. Put changing data in `state` and declare external inputs.
State supports finite scalars, plain tables with string/integer keys, engine
vectors/colors/quaternions/quads, engine Random, and immutable image/font/mesh/
matrix handles. Snapshots deeply copy mutable values and random state. Cycles,
shared table references, metatables, functions and unsupported userdata are
rejected with a field path. Nesting is limited to 64 levels. Immutable resources
retain their content identity across snapshots.

`k.random(seed)` provides deterministic `number()`, `integer()`, `boolean()` and
`fork(key)`, exclusive-upper-bound `range(min,max)`/`range_integer(min,max)`,
`chance(probability)`, `normal(mean,deviation)`, `choose(values,weights?)`,
`choose_multiple(values,amount)` and in-place `shuffle(values)`; global `math.random` is unavailable in simulation callbacks. Forks
are independent of consumed parent state. Construction starts Lua math.random
with a fixed seed; use ordered arrays when iteration order affects numeric results. See both complete examples in
[simulation](../examples/lua/simulation), including Game of Life and accumulation
with an animated input.

## Custom drawing and resources

Callbacks receive local coordinates. The native renderer handles transforms,
visibility, hierarchy, clipping and opacity. Draw receives copied properties and
state; changing those copies cannot mutate the engine. A drawing context expires
when the callback returns and cannot be kept for later use. Commands execute only
after a successful callback; save/layer operations must balance with restore.

2D supports `k.path()` with `move_to`, `line_to`, `quad_to`, `cubic_to` and
`close`. Context methods are `save`, `restore`, `layer(opacity)`, `translate`,
`scale`, `rotate(radians)`, `clip(path)`, `fill(path,paint)`,
`stroke(path,paint,width)`, `rect(x,y,w,h,paint)`, `circle(x,y,r,paint)`,
`image(image,x,y,w,h)` and `text(text,x,y,size,paint,font?)`. Paint is a color or
`{from={x,y},to={x,y},colors={color1,color2,...}}` for a linear gradient.
`k.font(path)` supplies a project TTF/OTF; omitted fonts use the system default.
Bounds are `{x,y,width,height}`. The [custom path scene](../examples/lua/2d/scenes/custom.lua)
uses a displaced box and manual opacity/position animation.

3D supports `k.mesh {vertices={{x,y,z},...},indices={0,1,2,...}}` with optional
normals, UVs and colors. Triangle indices are zero-based; normals are generated
when omitted. `ctx:mesh(mesh,matrix?)` and `ctx:instances(mesh,matrices)` use the
native geometry cache. Content-identical meshes reuse their identity; transform,
material and uniform edits do not rebuild geometry. `k.mat4()` is identity;
`k.mat4(values)` accepts 16 column-major numbers. Matrix `translate(vec3)`,
`scale(vec3)`, `rotate(quaternion)` and multiplication return new matrices.
Bounds are `{width,height,depth}` or displaced `{x,y,z,width,height,depth}`.
The [3D example](../examples/lua/3d/scenes/intro.lua) defines its own indexed mesh,
material and animated shader uniform.

`k.image(path)` freezes image content for the revision. Image/projection
constructors accept `source=image` or `source=canvas:texture()`. Canvas
`resolution={width,height}` is independent of project resolution. Textures and
canvas dependencies are checked for cycles and cross-scene references.

`k.image_shader {source=glsl}` or `{path="shader.glsl"}` supplies a GLSL 3.30 image
shader. `k.mesh_shader {fragment=glsl,vertex=optional_glsl}` accepts inline code,
or `fragment_path`/`vertex_path`. Object options include `shader`, `uniforms`,
`textures` and `shader_padding`. Animate uniforms through `obj:uniform("name")`.
The GLSL contract is the native one documented in the [README](../README.md).
Compilation failure preserves the active preview. Mesh/image/font/shader content
is immutable within an applied revision, including during export.

## Identity, reload and overrides

An explicit scene-local `id` preserves an entity across declaration insertion,
reordering and compatible edits. Unnamed objects use positional identities and
can be recreated by structural changes. Duplicate IDs fail preparation. Native
type or dynamic schema changes recreate only incompatible data; removed objects
are removed, and removed optional components do not linger.

Reload first builds an isolated candidate using the same ECS and scheduler,
replays compatible time, then validates native drawing/resources before applying
it between frames. Modules and resource paths determine affected scenes. The
apply step reconciles IDs, replaces tracks without accumulation, preserves
selection for surviving objects, and retains editor cameras and local time.
Comments and whitespace do not invalidate simulation logic. Visual properties
and draw changes retain compatible checkpoints when they are not declared inputs.
Update bodies, initial state/seed, FPS, lifetimes, scheduled operations and input
tracks invalidate the affected simulation and its dependents. Invalidation
currently replays that affected instance from its start; unrelated history is
preserved. Cancelled or failed candidates never replace active callbacks/state.

Inspector shows the script base, saved override and evaluated value separately.
For explicit IDs, editing a base persists to `kinematic/overrides.ron`, keyed by
scene path, object ID and property name. Overrides enter before animations are
constructed; they do not overwrite keyframes or freeze evaluated tracks.
Restore removes the saved entry. Incompatible/orphaned entries remain in the file
with diagnostics. Objects without IDs have explicitly temporary edits. Resolve
pending source edits before changing saved bases or event durations, so those
operations cannot silently discard a newer buffer. Event durations are stored in
`kinematic/events`, separate from sources and the manifest.

Export uses the last successfully applied preview revision, including applied
unsaved buffers. New edits remain pending until export finishes/cancels. Buffers
with errors can differ from that revision, as reported by Code status. Frames
advance by exactly `1/fps`; UI/selection overlays never enter output. Runtime or
drawing errors cancel export and remove the incomplete output through the native
exporter's failure path.

## Rust hosting and migration

Existing native scenes still use `#[scene]` and `SceneBuilder`. `SceneFactory`
now receives `&ProjectContext` before resolution/margins; the macro supplies the
new signature. `Project`/`Scene` names are owned strings. Use
`app().project_in(context,name,scenes)?` to choose an explicit native root.
Native projects retain `kinematic/project.ron`; Lua uses only `kinematic.ron`.

A host can open `LuaProject`, call `register::<MyObject>()?`, and run
`app().lua_project(project).run()`. Lua then calls
`s:native("my_builder", options)`. The same derive-generated metadata controls
field bindings, validation and `project.definitions()` for LuaLS. Duplicate
builder registration is rejected. Rust extensions remain appropriate for
external capabilities; custom geometry/simulations do not require them.

Code using the removed effects module, particle/morph types, text `.fade`/
`.morph`, or effect builder methods must switch to direct property tweens.
Custom draw structs using complete literals should supply the new optional
`logical_bounds` callback or use `..Default::default()`. Core/runtime contracts
remain independent of Lua VM storage and editor window ownership.

## Execution limits

Each scene VM has a 64 MiB memory limit and instruction-hook deadlines. Build/
full evaluation allows two seconds, drawing callbacks 100 ms; replay yields
between steps after about 8 ms. A single callback may consume its deadline.
Lua has no `io`, `os`, `package`, `debug`, arbitrary loaders or working-directory
changes. These restrictions prevent accidental hangs and filesystem access;
they do not make native Rust/GPU work interruptible. Images are limited to
64 MiB encoded and sixteen million decoded pixels, meshes to one million vertices
and three million indices, and drawing submissions/text have explicit caps.
Project/canvas dimensions are limited to 8192 pixels and project FPS to 1000.
Use project files only from trusted sources when invoking native extensions.

Reload timing is reported in Console: Lua construction, simulation replay,
application, GPU validation, geometry builds/uploads and the previous frame's
render time. Upload timings include CPU geometry construction and GL submission;
they do not measure asynchronous GPU completion.

See [validation evidence and platform limits](lua-validation.md).
