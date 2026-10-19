# Lua projects

Build the editor with `cargo build --release`. Lua 5.4 is embedded; no Lua
installation is needed. `cargo run --release` restores the current project,
or opens an empty workspace when none is saved. Select a project directly with:

```sh
cargo run --release -- examples/lua/2d
cargo run --release -- --new /tmp/my-animation "My animation"
cargo run --release -- --new /tmp/my-3d-animation "My 3D animation" --3d
cargo run --release -- --check examples/lua/simulation
cargo run --release -- --render examples/lua/3d
```

Rendering needs an OpenGL 3.3 desktop session and FFmpeg on `PATH`. `--check`
validates construction and evaluation without opening a window; it does not
validate GLSL compilation or GPU drawing. `--render` uses the application's
existing SDL/OpenGL/Skia/three-d resources and writes `output/<project name>.mp4`.
Existing exports are preserved by adding a numeric suffix to the new filename.

## Files and editing

`kinematic/project.ron` is the authoritative configuration:

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

Paths are relative to the project root (the parent of `kinematic/`), including
module imports, fonts, images, shader files, caches and output. Absolute paths, parent traversal and symlinks
that leave the project are rejected. `require("folder.module")` resolves
`folder/module.lua` or `folder/module/init.lua`. Imports are cached per revision;
cyclic imports are rejected. The global `k` is available in scenes and modules
and its engine API is read-only. Every scene listed in the manifest returns
exactly one `function(s)`; its displayed name is the file stem and its persistent
identity is the relative path. Modules may return functions or tables without
becoming scenes. The manifest alone selects scenes and their order.

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
control playback, fullscreen or application exit.

LuaLS is optional for execution. Install it yourself, set `KINEMATIC_LUALS` to
its executable, use an existing Neovim Mason installation, or put
`lua-language-server` on `PATH` (in that search order). Nothing is downloaded.
Each project owns an asynchronous server, with generated API definitions and
logs in `kinematic/cache/luals`. User LuaLS settings are not overwritten.
Completion opens automatically while typing, including after `.` and `:`;
Ctrl+Space requests it explicitly. Suggestions show member categories, signatures
and documentation. Use arrows to select, Tab/Enter to insert, and Esc to dismiss.
Vector, color, quaternion and property types retain their supported members through
chained calls. Custom `k.object` methods and `k.component` properties are inferred
from Lua source, including imported modules, without registering completion lists.
Members built dynamically from runtime strings may require type annotations.
Documentation is also available by hovering a symbol, and definition navigation
with Ctrl+click or F12. Signatures appear while typing
function arguments. The documentation popup stays in place while the pointer
moves into it, supports scrolling and text selection/copying, and closes after
the pointer leaves the symbol and popup. Buffers and version numbers
are sent to the server; stale replies are discarded. UTF-8/UTF-16 positions are
converted to the editor's character coordinates. Server failure is a diagnostic
and does not disable basic editing or execution.

## Scene API

```lua
return function(s)
    local ball = s:circle {
        id = "ball", radius = 24,
        position = k.vec2(-100, 0), fill = k.color("#60A5FA"),
    }
    s:world_2d():add(ball)
    ball.opacity:set(0)
    s:all(function()
        ball:to {position = k.vec2(100, 0)}:play(2, k.ease.linear)
        ball:to {opacity = 1}:play(0.5)
    end)
    s:wait(1)
end
```

Native constructor names and directly exposed properties come from the Rust
`Object` and `Trackable` macros. Rust `///` comments on objects, tracked fields,
and exported `#[lua]` methods are included automatically in LuaLS definitions.
Property defaults and numeric limits come from the runtime metadata, including
constructor presets. Write documentation next to the Rust declaration; do not
edit the generated cache. Shared Lua operations are documented in
`src/scripting/api.lua` and the property template `src/scripting/property.lua`. The generated file
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
`s:view("2d")` and `s:view("3d")` schedule an instantaneous view switch.
Native dimensions, transforms, opacity, materials, cameras, filters, lights and
layout use the same property metadata. Enum values use Rust variant names.

Builders accept complete values or partial channels: `position={x=-200}`,
`scale={y=2}` and `fill={a=0.5}` preserve omitted defaults. Vectors use `x/y/z`,
colors use `r/g/b/a`, and quads use `a/b/c/d`.

```lua
local movement = ball:by {position={x=100}}
movement:play(1)
movement:play(1)
ball:from {opacity={0,1},position={x={-200,200}}}
    :to {position={y=50}}:by {radius=10}:play(2,k.ease.cubic_out)
```

`to` declares absolute destinations; `by` declares numeric increments; `from`
uses exactly two endpoints for each property or channel. Complete pairs such as
`position={k.vec2(0,0),k.vec2(100,50)}` are supported. Omitted channels are not
animated. Overlapping declarations within a table or between chained calls are
errors. Property names, types and limits are validated.

Preparing a definition leaves the object and timeline unchanged. Each `play`
schedules an independent execution; implicit starts are resolved at its timeline
position. Reusing `to` repeats its absolute destination, while reusing `by`
applies the increment again. `play()` and `play(0)` are instantaneous;
`play(seconds,easing)` uses the given duration and optional native easing.
All chained destinations share that execution's duration and easing.

Use `object:get("position")` for reading and `object:prop("position.x")` for a
property handle, including names that collide with methods. Handles expose
`get`, `set` and `signal`; standalone `s:track(initial,id)` also supports
`to(value):play(...)`, `by(delta):play(...)` and `from({start,finish}):play(...)`.
Quaternion handles support `rotate(axis,radians):play(...)` to preserve full turns.

`s:wait(seconds)` advances the scope; `s:event(name)` adds a named editable wait.
`chain`, `all`, `parallel` and `loop` accept callbacks. `all` starts its children
together and advances by the longest duration; `chain` schedules sequentially;
`parallel` leaves the parent time unchanged; `loop` repeats a positive finite
cycle. Reusable Lua functions can schedule sequences.

`k.ease` uses lowercase snake case names such as `linear`, `cubic_out` and
`sine_in_out`. Nested repeats and ambiguous concurrent writes within repeats are
rejected by the native scheduler. Keyframes remain in timeline order.

Hierarchy queries include `children`, `get_child(index)`, `parent`,
`ancestor(type_name)` and `type_name`. Child positions are one-based.
`insert(child,index)`, `move_to`, `move_up`, `move_down`, `move_to_top` and
`move_to_bottom` preserve native ordering. `remove` and `remove_children` end
lifetimes; seeking backwards can restore the objects.

`bounds()` returns named local coordinates and dimensions; `box_size()` returns only size; `middle`, `left`, `right`, `top`, `bottom`
and corners return points in parent coordinates, including displaced bounds.
3D adds `front`, `back` and their edge/corner combinations, for example
`top_left_front`. Each anchor has `*_to(point)` returning the required local
position. Global queries are `global_position`, `global_rotation`,
`global_scale`, `global_matrix`, and 2D `global_opacity`. Image objects support
`get_pixel_color(vec2)`. A 3D canvas supports
`focus_camera{position=k.vec3(0,2,7),target=k.vec3(0,0,0)}:play(2)`.

`snapshot()` captures tracked fields, dynamic properties and uniforms.
`restore(snapshot):play(2)` uses an explicit snapshot; `save()` pushes a snapshot
and `restore():play(2)` pops it. `reset():play()` returns to initial builder values.
All three produce reusable animation definitions.

Objects can define methods using `k.object {name="Counter",components={...},
methods={increment=function(self,amount) return self:by{value=amount} end}}`.
Call `counter:increment(10):play(2)`. Fundamental object operations cannot be
replaced, and object methods are not copied to animation definitions.

Native methods are explicitly registered per type. An internal Rust handler
impl annotated with `#[lua_methods]` exports only `#[lua]` methods, generating
argument conversion and LuaLS signatures. Initially supported arguments are
`String`, `bool`, `f32`, `f64`, `i32`, `u32`, `i64` and `u64`; methods return
runtime tween `Task` definitions. Unsupported signatures fail at compilation.
Connect the generated method list in the native type registry once.
`text:fade("New text"):play(1)` switches the text at the execution start and
fades its opacity in. Advanced text, creation and morph transitions need their
own implementation.

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
    bounds = function() return {x = 0, y = -10, width = 200, height = 20} end,
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
read-only environment, isolated from construction-time library mutations.
The global `k` provides allowed value, Random, path, matrix and mesh constructors.
Inside callbacks, `require("utils.math")` loads helpers in this restricted
environment. Modules may return pure functions, scalars or immutable tables;
exported functions obey the same closure validation. Mutable captured state,
mutable exports, cycles and global writes are rejected. Put changing data in
`state` and declare external inputs. Module dependencies invalidate checkpoints
on reload. `inputs={}` supplies no inputs; omitted `inputs` uses own properties.
Subchannel inputs supply their scalar values.
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
Bounds are `{x=0,y=0,width=200,height=20}`. The [custom path scene](../examples/lua/2d/scenes/custom.lua)
uses a displaced box and manual opacity/position animation.

3D supports `k.mesh {vertices={{x,y,z},...},indices={1,2,3,...}}` with optional
normals, UVs and colors. Triangle indices are one-based; normals are generated
when omitted. `ctx:mesh(mesh,matrix?)` and `ctx:instances(mesh,matrices)` use the
native geometry cache. Content-identical meshes reuse their identity; transform,
material and uniform edits do not rebuild geometry. `k.mat4()` is identity;
`k.mat4(values)` accepts 16 column-major numbers. Matrix `translate(vec3)`,
`scale(vec3)`, `rotate(quaternion)` and multiplication return new matrices.
Bounds use named fields `{x=-1,y=-1,z=-1,width=2,height=2,depth=2}`.
The [3D example](../examples/lua/3d/scenes/intro.lua) defines its own indexed mesh,
material and animated shader uniform.

`k.image(path)` freezes image content for the revision. Image/projection
constructors accept `source=image` or `source=canvas:texture()`. Canvas
`resolution={width,height}` is independent of project resolution. Textures and
canvas dependencies are checked for cycles and cross-scene references.

`k.image_shader {source=glsl}` or `{path="shader.glsl"}` supplies a GLSL 3.30 image
shader. `k.mesh_shader {fragment=glsl,vertex=optional_glsl}` accepts inline code,
or `fragment_path`/`vertex_path`. Object options include `shader`, `uniforms`,
`textures` and `shader_padding`. Animate uniforms with `obj:to{uniforms={pulse=6}}:play(3)` or
`obj:from{uniforms={pulse={0,6}}}:play(3)`. Uniform names stay separate from
ordinary properties. `obj:uniform("name")` returns an explicit property handle.
The GLSL contract is described below.
Compilation failure preserves the active preview. Mesh/image/font/shader content
is immutable within an applied revision, including during export.

### GLSL contract

Image shaders are complete GLSL 3.30 fragment shaders. The engine supplies
`k_image`, pixel-sized `k_resolution`, scene seconds in `k_time`, and `k_alpha`.
UV `(0, 0)` is the bottom-left pixel. Input and output are premultiplied,
sRGB-encoded RGBA. Canvas samplers declared in `textures` follow the canvas
dependency order; cycles are rejected. `shader_padding` expands captures for
blur or glow.

Mesh shader sources omit `#version`. The standard vertex stage exposes
`k_world_position`, `k_normal` and `k_uv`. A custom vertex stage uses `position`
and optional `normal`/`uv_coordinates` attributes, and must actively use
`modelMatrix` and `viewProjection`. Optional engine uniforms are `normalMatrix`,
`viewMatrix`, `projectionMatrix`, `cameraPosition`, `sceneTime`, linear
`materialColor`, `hasNormal` and `hasUv`. Custom mesh shaders replace standard
material lighting. GPU deformation retains CPU geometry bounds; expand them
with `shader_padding` for selection and culling.

Group mesh shaders and uniforms are inherited by descendants. A child's own
shader takes precedence; compatible local uniforms override inherited ones.
Declare uniform bindings in object options before animating them. Uniforms
support booleans, numbers, vectors, quaternions, quads and colors; strings and
enums have no GLSL representation. Canvas textures remain fixed per revision.

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
`kinematic/scenes`, which holds scene metadata separately from Lua sources.
For example, `scenes/intro.lua` uses `kinematic/scenes/scenes/intro.ron`.

Export uses the last successfully applied preview revision, including applied
unsaved buffers. New edits remain pending until export finishes/cancels. Buffers
with errors can differ from that revision, as reported by Code status. Frames
advance by exactly `1/fps`; UI/selection overlays never enter output. Runtime or
drawing errors cancel export and remove the incomplete output through the native
exporter's failure path.

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


Reload timing is reported in Console: Lua construction, simulation replay,
application, GPU validation, geometry builds/uploads and the previous frame's
render time. Upload timings include CPU geometry construction and GL submission;
they do not measure asynchronous GPU completion.

See [validation evidence and platform limits](lua-validation.md).
