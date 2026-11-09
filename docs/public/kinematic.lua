---@meta

--- Position written as {line, column}. Both start at 1; columns count Unicode characters.
---@class Kinematic.CodePosition
---@field [integer] integer
local CodePosition = {}

--- Text interval. Prefer code:word, code:lines, code:find, or code:find_all to build it.
--- Manual form: {{start_line, start_column}, {end_line, end_column}}. The end is exclusive.
---@class Kinematic.CodeRange
---@field [integer] Kinematic.CodePosition
local CodeRange = {}

--- List mixing intervals, search results, and other lists. Missing search results are ignored.
---@class Kinematic.CodeRangeList
---@field [integer] Kinematic.CodeRange|Kinematic.CodeRangeList|nil
local CodeRangeList = {}

--- One interval or a list of intervals. Lists may be nested.
---@alias Kinematic.CodeRanges Kinematic.CodeRange|Kinematic.CodeRangeList

--- Vector, color, quaternion, or quad value.
---@alias Kinematic.Value Kinematic.Vec2|Kinematic.Vec3|Kinematic.Color|Kinematic.Quaternion|Kinematic.Quad

--- Reusable animation exposing only :play. Preparing it leaves the timeline unchanged.
---@class Kinematic.Animation
local Animation = {}

--- Schedules an independent execution at the current scene time.
--- Duration is in seconds and defaults to zero for an instantaneous change. Easing accepts only snake_case strings and defaults to "cubic_in_out".
---@param duration? number
---@param easing? Kinematic.Easing
function Animation:play(duration, easing) end

--- Typed handle to an animatable property or one of its scalar channels.
---@class Kinematic.Property<T>
local Property = {}

--- Scene object exposing properties, hierarchy operations, animations, and signals.
---@class Kinematic.Object
local Object = {}

--- Reads a named property value.
--- Names may include scalar channels, such as position.x. Use :prop to obtain a property handle instead.
---@param name string
---@return any
function Object:get(name) end

--- Returns a named property handle, including names that collide with methods.
--- Accepts channels such as position.x; the handle supports get, set, animation, and signals.
---@param name string
---@return Kinematic.Property
function Object:prop(name) end

--- Creates an animation toward absolute property destinations.
--- Accepts partial channels such as {position={x=100}}. Preparation leaves the timeline unchanged; call :play to schedule it.
---@param fields table
---@return Kinematic.Animation
function Object:to(fields) end

--- Creates an animation adding relative property increments.
--- Omitted channels remain unchanged. Each :play resolves its starting values at its own scheduled time.
---@param fields table
---@return Kinematic.Animation
function Object:by(fields) end

--- Creates an animation with explicit start and finish pairs for each property or channel.
--- Example: object:from {opacity={0,1}}:play(1). Overlapping destinations are errors.
---@param fields table
---@return Kinematic.Animation
function Object:from(fields) end

--- Attaches a child subtree at the current schedule time.
--- The receiver must support children, and both objects must belong to the same scene. Newly constructed objects stay inactive until attached.
---@param child Kinematic.Object
function Object:add(child) end

--- Moves this object to a new container for the whole timeline, preserving local properties and lifetime.
---@param node Kinematic.Object
function Object:reparent(node) end

--- Ends this object subtree lifetime at the current schedule time.
--- Entities and hierarchy edges are retained so seeking backward restores the object.
function Object:remove() end

--- Schedules a callback after scene evaluation while the object is active.
--- Receives the object and {time,index,dt}. Writes are temporary for that frame; the returned signal supports :stop().
---@param callback fun(object: Kinematic.Object, frame: {time:number,index:integer,dt:number})
---@return Kinematic.Signal
function Object:signal(callback) end

--- Scene construction and animation scheduling context passed to a scene function.
---@class Kinematic.Scene
local Scene = {}

--- Parameters for an immutable procedural audio resource; times are seconds and frequencies are Hz.
---@class Kinematic.AudioSynthOptions
---@field preset? "boink"|"pluck"|"click"|"impact"|"whoosh" Overridable starting parameters.
---@field waveform? "sine"|"triangle"|"noise" Default sine.
---@field frequency? number Initial frequency, 1–20000; default 440.
---@field end_frequency? number Final frequency of a linear sweep, 1–20000.
---@field attack? number Attack time, 0–10; default 0.004.
---@field decay? number Decay time, 0–10; default 0.08.
---@field sustain? number Sustain level, 0–1; default 0.3.
---@field hold? number Time at sustain level, 0–10; default 0.03.
---@field release? number Release time, 0–10; default 0.15.
---@field noise? number White noise blend, 0–1; default 0.
---@field overtone? number Second sine harmonic gain, 0–1; default 0.
---@field cutoff? number Low-pass cutoff, 20–20000; default 12000.
---@field seed? integer Deterministic noise seed, 0–4294967295; default 0.

--- Immutable audio content shared by players and occurrences.
---@class Kinematic.AudioResource
---@field duration number Read-only duration in seconds.

---@class Kinematic.AudioPlaybackOptions
---@field gain? number Volume, 0–1; default 0.2 on a new player.
---@field pan? number Stereo balance, -1–1; default 0.

---@class Kinematic.AudioOptions: Kinematic.AudioPlaybackOptions
---@field source Kinematic.AudioResource Shared immutable content.

--- Scene-local audio player with independent playback defaults.
---@class Kinematic.Audio
local Audio = {}

--- Schedules an occurrence without advancing the animator; overrides affect only this occurrence.
--- In simulation updates, use a declared audio input; the occurrence uses the step time.
---@param options? Kinematic.AudioPlaybackOptions
function Audio:play(options) end

--- Creates a silent player. Scheduled tails contribute to scene duration; repeat scopes are rejected.
---@param options Kinematic.AudioOptions
---@return Kinematic.Audio
function Scene:audio(options) end

--- Schedules an instantaneous switch between the built-in 2D and 3D world canvases.
--- The dimension must be "2d" or "3d".
---@param dimension "2d"|"3d"
function Scene:view(dimension) end

--- Advances the current scheduling scope by a nonnegative duration in seconds.
---@param seconds number
function Scene:wait(seconds) end

--- Adds a named editable wait at the current timeline position.
--- Its duration can be adjusted in the editor.
---@param name string
function Scene:event(name) end

--- Schedules the callback animations simultaneously.
--- The parent timeline advances by the longest child duration.
---@param schedule fun(scene: Kinematic.Scene)
function Scene:all(schedule) end

--- Schedules the callback animations sequentially on the current timeline.
---@param schedule fun(scene: Kinematic.Scene)
function Scene:chain(schedule) end

--- Schedules the callback without advancing or extending the parent timeline.
--- Animations inside the callback are sequential unless grouped with :all.
---@param schedule fun(scene: Kinematic.Scene)
function Scene:parallel(schedule) end

--- Repeats one positive finite animation cycle without advancing the parent timeline.
--- The callback executes once; its animations repeat until their lifetime ends. Nested loops and object creation, attachment, or removal within a cycle are rejected.
---@param schedule fun(scene: Kinematic.Scene)
function Scene:loop(schedule) end

--- Returns the built-in 2D canvas.
--- Attach 2D objects with scene:world_2d():add(object).
---@return Kinematic.Canvas2D
function Scene:world_2d() end

--- Returns the built-in 3D canvas.
--- Attach 3D objects with scene:world_3d():add(object).
---@return Kinematic.Canvas3D
function Scene:world_3d() end

--- Returns the internal root container shared by all scene contents.
--- Attach additional canvases here; use :world_2d() and :world_3d() for ordinary world objects.
---@return Kinematic.Object
function Scene:root() end

k = {}

--- Creates a silent synthesizer resource; the envelope must last 0.001–10 seconds.
---@param options Kinematic.AudioSynthOptions
---@return Kinematic.AudioResource
function k.audio_synth(options) end

--- Loads a project-relative WAV, MP3, OGG, or FLAC file using FFmpeg; creation does not play it.
---@param path string
---@return Kinematic.AudioResource
function k.audio_file(path) end


--- Creates a two-dimensional value with x and y components.
--- Use it for positions, dimensions, and other vector properties.
---@param x number
---@param y number
---@return Kinematic.Vec2
function k.vec2(x, y) end

--- Creates a three-dimensional value with x, y, and z components.
---@param x number
---@param y number
---@param z number
---@return Kinematic.Vec3
function k.vec3(x, y, z) end

--- Creates a color from RGB or RGBA numbers, or a six/eight-digit hexadecimal string.
--- Numeric components use the 0 to 1 convention; omitted alpha defaults to 1. Example: k.color("#60A5FA") or k.color(1,0,0,0.5).
---@param color string|number
---@param g? number
---@param b? number
---@param a? number
---@return Kinematic.Color
function k.color(color, g, b, a) end

--- Creates a quaternion from x, y, z, and w components.
--- Use k.axis_angle for an axis and an angle in radians.
---@param x number
---@param y number
---@param z number
---@param w number
---@return Kinematic.Quaternion
function k.quat(x, y, z, w) end

--- Frame information supplied to signals and simulation update callbacks.
---@class Kinematic.Frame
---@field time number Evaluation time in seconds.
---@field index integer Evaluation frame index.
---@field dt number Frame duration in seconds.

--- Scheduled callback handle. Call :stop to end its lifetime.
---@class Kinematic.Signal
local Signal = {}

--- Ends the signal lifetime at the current scheduling time.
function Signal:stop() end

--- Captured object property values accepted by :restore.
---@class Kinematic.Snapshot
--- Captures current tracked fields, dynamic properties, and shader uniforms.
--- Pass the returned snapshot to :restore(snapshot); it belongs to this object.
---@return Kinematic.Snapshot
function Object:snapshot() end

--- Pushes current property values onto this object snapshot stack.
--- A later :restore() without an argument pops the saved values.
function Object:save() end

--- Creates an animation toward an explicit snapshot, or pops the last :save() snapshot.
--- The snapshot must belong to this object. Call :play to schedule the restoration.
---@param snapshot? Kinematic.Snapshot
---@return Kinematic.Animation
function Object:restore(snapshot) end

--- Creates an animation toward the initial builder property values.
--- Call :play to schedule the reset.
---@return Kinematic.Animation
function Object:reset() end

--- Returns child object handles in hierarchy order.
---@return Kinematic.Object[]
function Object:children() end

--- Returns the parent object, or nil when there is no parent.
---@return Kinematic.Object?
function Object:parent() end

--- Returns the first ancestor whose :type_name() matches the supplied name.
--- Returns nil when no ancestor matches.
---@param type_name string
---@return Kinematic.Object?
function Object:ancestor(type_name) end

--- Returns the native object type name or the configured name of a scripted object.
---@return string
function Object:type_name() end

--- Returns the child at a one-based position in hierarchy order.
--- The index must refer to an existing child.
---@param index integer
---@return Kinematic.Object
function Object:get_child(index) end

--- Attaches a child subtree at the specified one-based hierarchy position.
--- Both objects must belong to the same scene and the receiver must support children.
---@param child Kinematic.Object
---@param index integer
function Object:insert(child, index) end

--- Ends the lifetimes of the child subtrees at the current schedule time.
--- Seeking backward restores their earlier visibility.
function Object:remove_children() end

--- Moves this object to a one-based position among its siblings.
---@param index integer
function Object:move_to(index) end

--- Moves this object one position toward the end of its sibling list.
function Object:move_up() end

--- Moves this object one position toward the start of its sibling list.
function Object:move_down() end

--- Moves this object to the end of its sibling list, above its siblings.
function Object:move_to_top() end

--- Moves this object to the start of its sibling list, below its siblings.
function Object:move_to_bottom() end

--- Returns the local bounding box with named coordinates and dimensions.
--- Two-dimensional objects return x, y, width, and height; 3D objects also return z and depth.
---@return {x:number,y:number,z?:number,width:number,height:number,depth?:number}
function Object:bounds() end

--- Returns the transformation matrix composed through the object hierarchy.
---@return Kinematic.Matrix
function Object:global_matrix() end

--- Returns a shader uniform property handle by name.
--- The uniform must be configured on the object.
---@param name string
---@return Kinematic.Property
function Object:uniform(name) end

--- Creates a standalone animatable property from an initial value.
--- An optional stable id preserves object identity across reloads. Supports get, set, to, by, from, and signal.
---@overload fun(self:Kinematic.Scene, initial:Kinematic.Vec2, id?:string):Kinematic.Vec2Property
---@overload fun(self:Kinematic.Scene, initial:Kinematic.Vec3, id?:string):Kinematic.Vec3Property
---@overload fun(self:Kinematic.Scene, initial:Kinematic.Color, id?:string):Kinematic.ColorProperty
---@overload fun(self:Kinematic.Scene, initial:Kinematic.Quaternion, id?:string):Kinematic.QuaternionProperty
---@overload fun(self:Kinematic.Scene, initial:Kinematic.Quad, id?:string):Kinematic.QuadProperty
---@generic T
---@param initial T
---@param id? string
---@return Kinematic.Property<T>
function Scene:track(initial, id) end

--- Instantiates a k.object definition with initial property and resource options.
--- Each instance receives independent state and stays inactive until attached to a group or canvas.
---@generic T
---@param definition T
---@param options table
---@return T
function Scene:object(definition, options) end

--- Reusable declaration of custom tracked fields for scripted objects.
---@class Kinematic.Component

--- Custom property schema; the required default value determines its type.
---@class Kinematic.PropertyDefinition
---@field default boolean|number|string|Kinematic.Value Required initial value; determines the property type.
---@field min? number Optional lower bound for a numeric property.
---@field max? number Optional upper bound for a numeric property.
---@field choices? string[] Allowed snake_case values for a string property.
--- Declares custom animatable fields from default values and optional numeric bounds or string choices.
--- The default determines each field type. Custom names must not shadow native properties.
---@param fields table<string,Kinematic.PropertyDefinition>
---@return Kinematic.Component
function k.component(fields) end

--- Reusable scripted object declaration created by k.object.
---@class Kinematic.ObjectDefinition

--- Scripted object callbacks, state, components, and methods.
---@class Kinematic.ObjectOptions
---@field dimension? 2|3 Spatial dimension, 2 or 3; defaults to 2.
---@field components? Kinematic.Component[] Custom tracked field declarations.
---@field state? table Initial state copied independently for each instance.
---@field inputs? table<string,Kinematic.Property|Kinematic.Object|Kinematic.Audio|string> Declared simulation dependencies. Omitted inputs use own properties; an empty table declares none.
---@field update? fun(state:table, frame:Kinematic.Frame, inputs:table) Fixed-step state update callback; external mutable upvalues are rejected.
---@field draw? fun(context:Kinematic.Draw2D|Kinematic.Draw3D, properties:table, state:table) Local drawing callback; reads evaluated properties and state.
---@field bounds? fun(properties:table, state:table):{x:number,y:number,z?:number,width:number,height:number,depth?:number} Returns local coordinates and dimensions for selection and layout.
---@field name? string Default display name of the scripted object.
---@field methods? table<string,function> Instance methods invoked with the object as self; fundamental operations cannot be replaced.
--- Declares a reusable scripted 2D or 3D object with components, state, callbacks, and methods.
--- Instantiate it with scene:object(definition, options). Drawing reads state; updates change state at fixed steps.
---@param definition Kinematic.ObjectOptions
---@return Kinematic.Script2D|Kinematic.Script3D
function k.object(definition) end

--- Creates a quaternion from a nonzero vec3 axis and a finite angle in radians.
--- The axis is normalized automatically.
---@param axis Kinematic.Vec3
---@param radians number
---@return Kinematic.Quaternion
function k.axis_angle(axis, radians) end

--- Creates a four-component value with a, b, c, and d channels.
--- For rectangle rounding, channels correspond to top-left, top-right, bottom-right, and bottom-left corners.
---@param a number
---@param b number
---@param c number
---@param d number
---@return Kinematic.Quad
function k.quad(a, b, c, d) end

--- Creates an immutable rectangle value from local coordinates and nonnegative dimensions.
--- This is a value helper; use scene:rect(options) to create a scene object.
---@param x number
---@param y number
---@param width number
---@param height number
---@return Kinematic.RectValue
function k.rect(x, y, width, height) end

--- Immutable rectangle value with coordinate, dimension, and anchor queries.
---@class Kinematic.RectValue
---@field x number X component or left coordinate.
---@field y number Y component or bottom coordinate.
---@field width number Rectangle width in local coordinates.
---@field height number Rectangle height in local coordinates.
local RectValue = {}

--- Deterministic mutable random generator that can be preserved in simulation state.
---@class Kinematic.Random
local Random = {}

--- Creates a deterministic random generator from a nonnegative integer seed.
--- Store it in simulation state to preserve its sequence during seek replay.
---@param seed integer
---@return Kinematic.Random
function k.random(seed) end

--- Returns a uniformly distributed number from 0 inclusive to 1 exclusive.
---@return number
function Random:number() end

--- Returns a uniformly distributed unsigned 32-bit integer.
---@return integer
function Random:integer() end

--- Returns a random boolean with equal probability for each value.
---@return boolean
function Random:boolean() end

--- Creates an independent deterministic generator derived from this seed and key.
--- Does not consume the parent generator sequence.
---@param key integer
---@return Kinematic.Random
function Random:fork(key) end

--- Mutable path used by local 2D drawing operations.
---@class Kinematic.Path
local Path = {}

--- Creates a mutable drawing path.
--- Use move_to, line_to, quad_to, cubic_to, and close to build its geometry.
---@return Kinematic.Path
function k.path() end

--- Starts a contour at the supplied local coordinates.
---@param x number
---@param y number
function Path:move_to(x, y) end

--- Adds a straight segment from the current point to x, y.
---@param x number
---@param y number
function Path:line_to(x, y) end

--- Adds a quadratic Bezier segment using x1, y1 as control point and x2, y2 as endpoint.
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
function Path:quad_to(x1, y1, x2, y2) end

--- Adds a cubic Bezier segment with two control points and x3, y3 as endpoint.
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param x3 number
---@param y3 number
function Path:cubic_to(x1, y1, x2, y2, x3, y3) end

--- Closes the current contour by connecting it to its starting point.
function Path:close() end

--- Immutable column-major 4x4 transformation matrix.
---@class Kinematic.Matrix
---@operator mul(Kinematic.Matrix): Kinematic.Matrix
local Matrix = {}

--- Creates a matrix from sixteen column-major numbers, or the identity when omitted.
---@param columns? number[]
---@return Kinematic.Matrix
function k.mat4(columns) end

--- Returns this matrix multiplied by a translation matrix.
--- The translation must be a vec3; the original matrix is unchanged.
---@param value Kinematic.Value
---@return Kinematic.Matrix
function Matrix:translate(value) end

--- Returns this matrix multiplied by a scale matrix.
--- The scale must be a vec3; the original matrix is unchanged.
---@param value Kinematic.Value
---@return Kinematic.Matrix
function Matrix:scale(value) end

--- Returns this matrix multiplied by a quaternion rotation matrix.
--- The original matrix is unchanged.
---@param quaternion Kinematic.Value
---@return Kinematic.Matrix
function Matrix:rotate(quaternion) end

--- Immutable indexed triangle geometry used by 3D drawing.
---@class Kinematic.Mesh
--- Creates an immutable indexed triangle mesh with one-based indices.
--- Vertices are vec3 number arrays; optional normals, UVs, and colors must match the vertex count. Normals are generated when omitted.
---@param data {vertices:number[][],indices:integer[],normals?:number[][],uvs?:number[][],colors?:Kinematic.Value[]}
---@return Kinematic.Mesh
function k.mesh(data) end

--- Decoded image resource loaded from a project-relative path.
---@class Kinematic.Image

--- Font resource loaded from a project-relative path.
---@class Kinematic.Font

--- Image or mesh shader resource with typed uniform inputs.
---@class Kinematic.Shader

--- Live texture produced by a canvas or projection.
---@class Kinematic.Texture
--- Loads an image resource from a path relative to the project directory.
--- Use the returned handle as an image object source or in a drawing callback.
---@param path string
---@return Kinematic.Image
function k.image(path) end

--- Loads a font resource from a path relative to the project directory.
---@param path string
---@return Kinematic.Font
function k.font(path) end

--- Creates a 2D image shader from inline source or a project-relative path.
--- When both are supplied, the file path takes precedence.
---@param options {source?:string,path?:string}
---@return Kinematic.Shader
function k.image_shader(options) end

--- Creates a 3D mesh shader with a required fragment source and an optional vertex source.
--- Sources may be inline or project-relative files; file paths take precedence.
---@param options {fragment?:string,fragment_path?:string,vertex?:string,vertex_path?:string}
---@return Kinematic.Shader
function k.mesh_shader(options) end

--- Optional resource configuration shared by native object constructors.
---@class Kinematic.ResourceOptions
---@field source? Kinematic.Image|Kinematic.Texture Image resource for image objects, or live canvas texture for projection objects.
---@field font? Kinematic.Font Project font for Text2D, Text3D, or Code; LaTeX uses its mathematical fonts.
---@field shader? Kinematic.Shader Image or mesh shader resource.
---@field uniforms? table<string,boolean|number|Kinematic.Value> Initial typed shader uniform values; requires shader.
---@field textures? table<string,Kinematic.Texture> Named live textures for image shaders; bindings require shader.
---@field resolution? integer[] Required width and height for additional canvases, each 1–8192 pixels.
---@field pixels_per_unit? number Image/projection pixels per local 3D unit; positive and finite, default 256.
---@field segments? integer Fixed sphere subdivision count, 3–256; default 32.
---@field shader_padding? number Extra image capture or mesh bounds padding; finite and nonnegative, default 0.
---@alias Kinematic.Paint Kinematic.Value|{from:number[],to:number[],colors:Kinematic.Value[]}

--- Local 2D drawing context available only during scripted draw callbacks.
---@class Kinematic.Draw2D
local Draw2D = {}

--- Saves the drawing transform and clip for a later :restore().
function Draw2D:save() end

--- Restores the most recently saved drawing state or closes a saved opacity layer.
function Draw2D:restore() end

--- Begins a composited layer with the supplied opacity.
--- Call :restore() to composite all drawings in that layer once.
---@param opacity number
function Draw2D:layer(opacity) end

--- Translates subsequent drawing operations in local coordinates.
---@param x number
---@param y number
function Draw2D:translate(x, y) end

--- Scales subsequent drawing operations on each axis.
---@param x number
---@param y number
function Draw2D:scale(x, y) end

--- Rotates subsequent drawing operations by an angle in radians.
---@param radians number
function Draw2D:rotate(radians) end

--- Intersects the current clip with a drawing path.
---@param path Kinematic.Path
function Draw2D:clip(path) end

--- Fills a path with a color or linear gradient paint.
---@param path Kinematic.Path
---@param paint Kinematic.Paint
function Draw2D:fill(path, paint) end

--- Outlines a path with a color or linear gradient and a nonnegative width.
---@param path Kinematic.Path
---@param paint Kinematic.Paint
---@param width number
function Draw2D:stroke(path, paint, width) end

--- Fills a rectangle in local coordinates with a color or gradient.
---@param x number
---@param y number
---@param width number
---@param height number
---@param paint Kinematic.Paint
function Draw2D:rect(x, y, width, height, paint) end

--- Fills a circle in local coordinates with a color or gradient.
---@param x number
---@param y number
---@param radius number
---@param paint Kinematic.Paint
function Draw2D:circle(x, y, radius, paint) end

--- Draws an image into the supplied local rectangle.
---@param image Kinematic.Image
---@param x number
---@param y number
---@param width number
---@param height number
function Draw2D:image(image, x, y, width, height) end

--- Draws text at local coordinates with a size, paint, and optional font.
--- When omitted, a default system drawing font is used.
---@param text string
---@param x number
---@param y number
---@param size number
---@param paint Kinematic.Paint
---@param font? Kinematic.Font
function Draw2D:text(text, x, y, size, paint, font) end

--- Local 3D drawing context available only during scripted draw callbacks.
---@class Kinematic.Draw3D
local Draw3D = {}

--- Draws a mesh using the optional local transformation matrix.
--- Drawing callbacks must not mutate animation or simulation state.
---@param mesh Kinematic.Mesh
---@param matrix? Kinematic.Matrix
function Draw3D:mesh(mesh, matrix) end

--- Draws the same mesh at each local transformation matrix.
---@param mesh Kinematic.Mesh
---@param matrices Kinematic.Matrix[]
function Draw3D:instances(mesh, matrices) end

--- Returns a uniformly distributed number in [min, max).
--- Both bounds must be finite and min must be less than max.
---@param min number
---@param max number
---@return number
function Random:range(min, max) end

--- Returns a uniformly distributed integer in [min, max).
--- The minimum must be less than the exclusive maximum.
---@param min integer
---@param max integer
---@return integer
function Random:range_integer(min, max) end

--- Returns true with the supplied probability from 0 to 1 inclusive.
---@param probability number
---@return boolean
function Random:chance(probability) end

--- Returns a normally distributed number with the supplied mean and standard deviation.
---@param mean number
---@param deviation number
---@return number
function Random:normal(mean, deviation) end

--- Chooses one array entry, optionally using matching nonnegative weights.
--- Returns nil for an empty array, invalid weights, or all-zero weights.
---@generic T
---@param values T[]
---@param weights? number[]
---@return T?
function Random:choose(values, weights) end

--- Chooses up to amount distinct array entries without changing the input array.
---@generic T
---@param values T[]
---@param amount integer
---@return T[]
function Random:choose_multiple(values, amount) end

--- Randomly permutes the array in place and returns the same table.
---@generic T
---@param values T[]
---@return T[]
function Random:shuffle(values) end

--- Returns the middle point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:middle() end

--- Returns the top point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:top() end

--- Returns the bottom point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:bottom() end

--- Returns the left point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:left() end

--- Returns the right point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:right() end

--- Returns the top_left point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:top_left() end

--- Returns the bottom_right point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:bottom_right() end

--- Returns the top_right point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:top_right() end

--- Returns the bottom_left point of this rectangle as a vec2.
---@return Kinematic.Vec2
function RectValue:bottom_left() end

--- Reads the current value of this property or scalar channel.
---@generic T
---@param self Kinematic.Property<T>
---@return T
function Property:get() end

--- Sets the property or scalar channel immediately during construction.
--- Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.
---@generic T
---@param self Kinematic.Property<T>
---@param value T
function Property:set(value) end

--- Creates an animation toward an absolute value.
--- Preparing the animation does not schedule it; call :play(duration, easing).
---@generic T
---@param self Kinematic.Property<T>
---@param value T
---@return Kinematic.Animation
function Property:to(value) end

--- Creates an animation adding a relative increment.
--- Repeated :play calls apply the increment at each scheduled start.
---@generic T
---@param self Kinematic.Property<T>
---@param delta T
---@return Kinematic.Animation
function Property:by(delta) end

--- Creates an animation with explicit {start, finish} endpoints.
--- Both endpoints must match the property type. Call :play to schedule it.
---@generic T
---@param self Kinematic.Property<T>
---@param pair T[]
---@return Kinematic.Animation
function Property:from(pair) end

--- Schedules a callback receiving this property and the frame after scene evaluation.
--- Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.
---@generic T
---@param self Kinematic.Property<T>
---@param callback fun(property:Kinematic.Property<T>, frame:Kinematic.Frame)
---@return Kinematic.Signal
function Property:signal(callback) end

--- Immutable Vec2 value.
---@class Kinematic.Vec2
---@field x number The x component.
---@field y number The y component.
---@operator add(Kinematic.Vec2): Kinematic.Vec2
---@operator sub(Kinematic.Vec2): Kinematic.Vec2
---@operator mul(number): Kinematic.Vec2
---@operator div(number): Kinematic.Vec2
---@operator unm: Kinematic.Vec2
local Vec2 = {}

--- Interpolates between compatible values.
---@param other Kinematic.Vec2
---@param t number Interpolation factor.
---@return Kinematic.Vec2
function Vec2:lerp(other, t) end

--- Returns the dot product.
---@param other Kinematic.Vec2
---@return number
function Vec2:dot(other) end

--- Returns the vector magnitude.
---@return number
function Vec2:length() end

--- Returns a normalized value. Zero vectors remain zero.
---@return Kinematic.Vec2
function Vec2:normalized() end

--- Animatable Vec2 property.
---@class Kinematic.Vec2Property: Kinematic.Property<Kinematic.Vec2>
---@field x Kinematic.Property<number> Animatable x component.
---@field y Kinematic.Property<number> Animatable y component.
local Vec2Property = {}

--- Reads the current value of this property or scalar channel.
---@param self Kinematic.Vec2Property
---@return Kinematic.Vec2
function Vec2Property:get() end

--- Sets the property or scalar channel immediately during construction.
--- Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.
---@param self Kinematic.Vec2Property
---@param value Kinematic.Vec2
function Vec2Property:set(value) end

--- Creates an animation toward an absolute value.
--- Preparing the animation does not schedule it; call :play(duration, easing).
---@param self Kinematic.Vec2Property
---@param value Kinematic.Vec2
---@return Kinematic.Animation
function Vec2Property:to(value) end

--- Creates an animation adding a relative increment.
--- Repeated :play calls apply the increment at each scheduled start.
---@param self Kinematic.Vec2Property
---@param delta Kinematic.Vec2
---@return Kinematic.Animation
function Vec2Property:by(delta) end

--- Creates an animation with explicit {start, finish} endpoints.
--- Both endpoints must match the property type. Call :play to schedule it.
---@param self Kinematic.Vec2Property
---@param pair Kinematic.Vec2[]
---@return Kinematic.Animation
function Vec2Property:from(pair) end

--- Schedules a callback receiving this property and the frame after scene evaluation.
--- Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.
---@param self Kinematic.Vec2Property
---@param callback fun(property:Kinematic.Vec2Property, frame:Kinematic.Frame)
---@return Kinematic.Signal
function Vec2Property:signal(callback) end


--- Immutable Vec3 value.
---@class Kinematic.Vec3
---@field x number The x component.
---@field y number The y component.
---@field z number The z component.
---@operator add(Kinematic.Vec3): Kinematic.Vec3
---@operator sub(Kinematic.Vec3): Kinematic.Vec3
---@operator mul(number): Kinematic.Vec3
---@operator div(number): Kinematic.Vec3
---@operator unm: Kinematic.Vec3
local Vec3 = {}

--- Interpolates between compatible values.
---@param other Kinematic.Vec3
---@param t number Interpolation factor.
---@return Kinematic.Vec3
function Vec3:lerp(other, t) end

--- Returns the dot product.
---@param other Kinematic.Vec3
---@return number
function Vec3:dot(other) end

--- Returns the vector magnitude.
---@return number
function Vec3:length() end

--- Returns a normalized value. Zero vectors remain zero.
---@return Kinematic.Vec3
function Vec3:normalized() end

--- Returns the cross product.
---@param other Kinematic.Vec3
---@return Kinematic.Vec3
function Vec3:cross(other) end

--- Animatable Vec3 property.
---@class Kinematic.Vec3Property: Kinematic.Property<Kinematic.Vec3>
---@field x Kinematic.Property<number> Animatable x component.
---@field y Kinematic.Property<number> Animatable y component.
---@field z Kinematic.Property<number> Animatable z component.
local Vec3Property = {}

--- Reads the current value of this property or scalar channel.
---@param self Kinematic.Vec3Property
---@return Kinematic.Vec3
function Vec3Property:get() end

--- Sets the property or scalar channel immediately during construction.
--- Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.
---@param self Kinematic.Vec3Property
---@param value Kinematic.Vec3
function Vec3Property:set(value) end

--- Creates an animation toward an absolute value.
--- Preparing the animation does not schedule it; call :play(duration, easing).
---@param self Kinematic.Vec3Property
---@param value Kinematic.Vec3
---@return Kinematic.Animation
function Vec3Property:to(value) end

--- Creates an animation adding a relative increment.
--- Repeated :play calls apply the increment at each scheduled start.
---@param self Kinematic.Vec3Property
---@param delta Kinematic.Vec3
---@return Kinematic.Animation
function Vec3Property:by(delta) end

--- Creates an animation with explicit {start, finish} endpoints.
--- Both endpoints must match the property type. Call :play to schedule it.
---@param self Kinematic.Vec3Property
---@param pair Kinematic.Vec3[]
---@return Kinematic.Animation
function Vec3Property:from(pair) end

--- Schedules a callback receiving this property and the frame after scene evaluation.
--- Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.
---@param self Kinematic.Vec3Property
---@param callback fun(property:Kinematic.Vec3Property, frame:Kinematic.Frame)
---@return Kinematic.Signal
function Vec3Property:signal(callback) end


--- Immutable Color value.
---@class Kinematic.Color
---@field r number The r component.
---@field g number The g component.
---@field b number The b component.
---@field a number The a component.
---@operator add(Kinematic.Color): Kinematic.Color
---@operator sub(Kinematic.Color): Kinematic.Color
---@operator mul(number): Kinematic.Color
---@operator div(number): Kinematic.Color
---@operator unm: Kinematic.Color
local Color = {}

--- Interpolates between compatible values.
---@param other Kinematic.Color
---@param t number Interpolation factor.
---@return Kinematic.Color
function Color:lerp(other, t) end

--- Animatable Color property.
---@class Kinematic.ColorProperty: Kinematic.Property<Kinematic.Color>
---@field r Kinematic.Property<number> Animatable r component.
---@field g Kinematic.Property<number> Animatable g component.
---@field b Kinematic.Property<number> Animatable b component.
---@field a Kinematic.Property<number> Animatable a component.
local ColorProperty = {}

--- Reads the current value of this property or scalar channel.
---@param self Kinematic.ColorProperty
---@return Kinematic.Color
function ColorProperty:get() end

--- Sets the property or scalar channel immediately during construction.
--- Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.
---@param self Kinematic.ColorProperty
---@param value Kinematic.Color
function ColorProperty:set(value) end

--- Creates an animation toward an absolute value.
--- Preparing the animation does not schedule it; call :play(duration, easing).
---@param self Kinematic.ColorProperty
---@param value Kinematic.Color
---@return Kinematic.Animation
function ColorProperty:to(value) end

--- Creates an animation adding a relative increment.
--- Repeated :play calls apply the increment at each scheduled start.
---@param self Kinematic.ColorProperty
---@param delta Kinematic.Color
---@return Kinematic.Animation
function ColorProperty:by(delta) end

--- Creates an animation with explicit {start, finish} endpoints.
--- Both endpoints must match the property type. Call :play to schedule it.
---@param self Kinematic.ColorProperty
---@param pair Kinematic.Color[]
---@return Kinematic.Animation
function ColorProperty:from(pair) end

--- Schedules a callback receiving this property and the frame after scene evaluation.
--- Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.
---@param self Kinematic.ColorProperty
---@param callback fun(property:Kinematic.ColorProperty, frame:Kinematic.Frame)
---@return Kinematic.Signal
function ColorProperty:signal(callback) end


--- Immutable Quaternion value.
---@class Kinematic.Quaternion
---@field x number The x component.
---@field y number The y component.
---@field z number The z component.
---@field w number The w component.
---@operator mul(Kinematic.Quaternion): Kinematic.Quaternion
---@operator mul(Kinematic.Vec3): Kinematic.Vec3
local Quaternion = {}

--- Interpolates between compatible values.
---@param other Kinematic.Quaternion
---@param t number Interpolation factor.
---@return Kinematic.Quaternion
function Quaternion:lerp(other, t) end

--- Returns a normalized value. Zero vectors remain zero.
---@return Kinematic.Quaternion
function Quaternion:normalized() end

--- Animatable Quaternion property.
---@class Kinematic.QuaternionProperty: Kinematic.Property<Kinematic.Quaternion>
local QuaternionProperty = {}

--- Reads the current value of this property or scalar channel.
---@param self Kinematic.QuaternionProperty
---@return Kinematic.Quaternion
function QuaternionProperty:get() end

--- Sets the property or scalar channel immediately during construction.
--- Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.
---@param self Kinematic.QuaternionProperty
---@param value Kinematic.Quaternion
function QuaternionProperty:set(value) end

--- Creates an animation toward an absolute value.
--- Preparing the animation does not schedule it; call :play(duration, easing).
---@param self Kinematic.QuaternionProperty
---@param value Kinematic.Quaternion
---@return Kinematic.Animation
function QuaternionProperty:to(value) end

--- Creates an animation adding a relative increment.
--- Repeated :play calls apply the increment at each scheduled start.
---@param self Kinematic.QuaternionProperty
---@param delta Kinematic.Quaternion
---@return Kinematic.Animation
function QuaternionProperty:by(delta) end

--- Creates an animation with explicit {start, finish} endpoints.
--- Both endpoints must match the property type. Call :play to schedule it.
---@param self Kinematic.QuaternionProperty
---@param pair Kinematic.Quaternion[]
---@return Kinematic.Animation
function QuaternionProperty:from(pair) end

--- Schedules a callback receiving this property and the frame after scene evaluation.
--- Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.
---@param self Kinematic.QuaternionProperty
---@param callback fun(property:Kinematic.QuaternionProperty, frame:Kinematic.Frame)
---@return Kinematic.Signal
function QuaternionProperty:signal(callback) end

--- Creates a quaternion rotation preserving full turns. Call :play to schedule it.
---@param axis Kinematic.Vec3
---@param radians number
---@return Kinematic.Animation
function QuaternionProperty:rotate(axis, radians) end

--- Immutable Quad value.
---@class Kinematic.Quad
---@field a number The a component.
---@field b number The b component.
---@field c number The c component.
---@field d number The d component.
---@operator mul(number): Kinematic.Quad
---@operator div(number): Kinematic.Quad
---@operator unm: Kinematic.Quad
local Quad = {}

--- Interpolates between compatible values.
---@param other Kinematic.Quad
---@param t number Interpolation factor.
---@return Kinematic.Quad
function Quad:lerp(other, t) end

--- Animatable Quad property.
---@class Kinematic.QuadProperty: Kinematic.Property<Kinematic.Quad>
---@field a Kinematic.Property<number> Animatable a component.
---@field b Kinematic.Property<number> Animatable b component.
---@field c Kinematic.Property<number> Animatable c component.
---@field d Kinematic.Property<number> Animatable d component.
local QuadProperty = {}

--- Reads the current value of this property or scalar channel.
---@param self Kinematic.QuadProperty
---@return Kinematic.Quad
function QuadProperty:get() end

--- Sets the property or scalar channel immediately during construction.
--- Inside a signal, writes apply only to the evaluated frame. Values must match the property type and are constrained by its limits.
---@param self Kinematic.QuadProperty
---@param value Kinematic.Quad
function QuadProperty:set(value) end

--- Creates an animation toward an absolute value.
--- Preparing the animation does not schedule it; call :play(duration, easing).
---@param self Kinematic.QuadProperty
---@param value Kinematic.Quad
---@return Kinematic.Animation
function QuadProperty:to(value) end

--- Creates an animation adding a relative increment.
--- Repeated :play calls apply the increment at each scheduled start.
---@param self Kinematic.QuadProperty
---@param delta Kinematic.Quad
---@return Kinematic.Animation
function QuadProperty:by(delta) end

--- Creates an animation with explicit {start, finish} endpoints.
--- Both endpoints must match the property type. Call :play to schedule it.
---@param self Kinematic.QuadProperty
---@param pair Kinematic.Quad[]
---@return Kinematic.Animation
function QuadProperty:from(pair) end

--- Schedules a callback receiving this property and the frame after scene evaluation.
--- Writes are temporary for that frame. Signals cannot construct objects or schedule animations; call :stop on the returned handle to end it.
---@param self Kinematic.QuadProperty
---@param callback fun(property:Kinematic.QuadProperty, frame:Kinematic.Frame)
---@return Kinematic.Signal
function QuadProperty:signal(callback) end


--- Easing curve names accepted by Animation:play. The default is "cubic_in_out".
---@alias Kinematic.Easing "linear"|"quad_in"|"quad_out"|"quad_in_out"|"cubic_in"|"cubic_out"|"cubic_in_out"|"quart_in"|"quart_out"|"quart_in_out"|"quint_in"|"quint_out"|"quint_in_out"|"sine_in"|"sine_out"|"sine_in_out"|"expo_in"|"expo_out"|"expo_in_out"|"circ_in"|"circ_out"|"circ_in_out"|"back_in"|"back_out"|"back_in_out"|"elastic_in"|"elastic_out"|"elastic_in_out"|"bounce_in"|"bounce_out"|"bounce_in_out"

--- Shared spatial queries for compatible objects.
---@class Kinematic.Spatial2D: Kinematic.Object
local Spatial2D = {}

--- Returns the local bounding box dimensions as a vector.
---@return Kinematic.Value
function Spatial2D:box_size() end

--- Returns the position composed through the object hierarchy.
---@return Kinematic.Value
function Spatial2D:global_position() end

--- Returns the scale composed through the object hierarchy.
---@return Kinematic.Value
function Spatial2D:global_scale() end

--- Returns the rotation composed through the object hierarchy.
--- The result is radians for a 2D object or a quaternion for a 3D object.
---@return Kinematic.Value|number
function Spatial2D:global_rotation() end

--- Returns opacity composed through the hierarchy. Requires a 2D object.
---@return number
function Spatial2D:global_opacity() end

--- Returns the middle anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:middle() end

--- Returns the local position that places the middle anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:middle_to(position) end

--- Returns the left anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:left() end

--- Returns the local position that places the left anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:left_to(position) end

--- Returns the right anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:right() end

--- Returns the local position that places the right anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:right_to(position) end

--- Returns the top anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:top() end

--- Returns the local position that places the top anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:top_to(position) end

--- Returns the bottom anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:bottom() end

--- Returns the local position that places the bottom anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:bottom_to(position) end

--- Returns the top_left anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:top_left() end

--- Returns the local position that places the top_left anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:top_left_to(position) end

--- Returns the top_right anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:top_right() end

--- Returns the local position that places the top_right anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:top_right_to(position) end

--- Returns the bottom_left anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:bottom_left() end

--- Returns the local position that places the bottom_left anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:bottom_left_to(position) end

--- Returns the bottom_right anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial2D:bottom_right() end

--- Returns the local position that places the bottom_right anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial2D:bottom_right_to(position) end

--- Shared spatial queries for compatible objects.
---@class Kinematic.Spatial3D: Kinematic.Object
local Spatial3D = {}

--- Returns the local bounding box dimensions as a vector.
---@return Kinematic.Value
function Spatial3D:box_size() end

--- Returns the position composed through the object hierarchy.
---@return Kinematic.Value
function Spatial3D:global_position() end

--- Returns the scale composed through the object hierarchy.
---@return Kinematic.Value
function Spatial3D:global_scale() end

--- Returns the rotation composed through the object hierarchy.
--- The result is radians for a 2D object or a quaternion for a 3D object.
---@return Kinematic.Value|number
function Spatial3D:global_rotation() end

--- Returns the middle anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:middle() end

--- Returns the local position that places the middle anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:middle_to(position) end

--- Returns the left anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:left() end

--- Returns the local position that places the left anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:left_to(position) end

--- Returns the right anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:right() end

--- Returns the local position that places the right anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:right_to(position) end

--- Returns the top anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top() end

--- Returns the local position that places the top anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_to(position) end

--- Returns the bottom anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom() end

--- Returns the local position that places the bottom anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_to(position) end

--- Returns the top_left anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_left() end

--- Returns the local position that places the top_left anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_left_to(position) end

--- Returns the top_right anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_right() end

--- Returns the local position that places the top_right anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_right_to(position) end

--- Returns the bottom_left anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_left() end

--- Returns the local position that places the bottom_left anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_left_to(position) end

--- Returns the bottom_right anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_right() end

--- Returns the local position that places the bottom_right anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_right_to(position) end

--- Returns the front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:front() end

--- Returns the local position that places the front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:front_to(position) end

--- Returns the back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:back() end

--- Returns the local position that places the back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:back_to(position) end

--- Returns the left_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:left_front() end

--- Returns the local position that places the left_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:left_front_to(position) end

--- Returns the right_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:right_front() end

--- Returns the local position that places the right_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:right_front_to(position) end

--- Returns the top_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_front() end

--- Returns the local position that places the top_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_front_to(position) end

--- Returns the bottom_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_front() end

--- Returns the local position that places the bottom_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_front_to(position) end

--- Returns the top_left_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_left_front() end

--- Returns the local position that places the top_left_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_left_front_to(position) end

--- Returns the top_right_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_right_front() end

--- Returns the local position that places the top_right_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_right_front_to(position) end

--- Returns the bottom_left_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_left_front() end

--- Returns the local position that places the bottom_left_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_left_front_to(position) end

--- Returns the bottom_right_front anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_right_front() end

--- Returns the local position that places the bottom_right_front anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_right_front_to(position) end

--- Returns the left_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:left_back() end

--- Returns the local position that places the left_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:left_back_to(position) end

--- Returns the right_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:right_back() end

--- Returns the local position that places the right_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:right_back_to(position) end

--- Returns the top_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_back() end

--- Returns the local position that places the top_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_back_to(position) end

--- Returns the bottom_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_back() end

--- Returns the local position that places the bottom_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_back_to(position) end

--- Returns the top_left_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_left_back() end

--- Returns the local position that places the top_left_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_left_back_to(position) end

--- Returns the top_right_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_right_back() end

--- Returns the local position that places the top_right_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:top_right_back_to(position) end

--- Returns the bottom_left_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_left_back() end

--- Returns the local position that places the bottom_left_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_left_back_to(position) end

--- Returns the bottom_right_back anchor of the bounding box in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_right_back() end

--- Returns the local position that places the bottom_right_back anchor at position.
---@param position Kinematic.Value Target position in parent coordinates.
---@return Kinematic.Value
function Spatial3D:bottom_right_back_to(position) end

--- Built-in circular scene object.
---@class Kinematic.Circle: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Circle, callback:fun(object:Kinematic.Circle, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field radius Kinematic.Property<number> Radius in local coordinates. Default: 128. Minimum: 0.
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Circle = {}

--- Initial properties and resources for circle.
---@class Kinematic.CircleOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field radius? number Radius in local coordinates. Default: 128. Minimum: 0.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in circular scene object.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.CircleOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Circle object Handle exposing animatable properties and object methods.
function Scene:circle(options) end

--- Built-in elliptical scene object.
---@class Kinematic.Ellipse: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Ellipse, callback:fun(object:Kinematic.Ellipse, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field radius Kinematic.Vec2Property Horizontal and vertical radii in local coordinates. Default: [128, 64].
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Ellipse = {}

--- Initial properties and resources for ellipse.
---@class Kinematic.EllipseOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field radius? Kinematic.Vec2|{x?:number,y?:number} Horizontal and vertical radii in local coordinates. Default: [128, 64].
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in elliptical scene object.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.EllipseOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Ellipse object Handle exposing animatable properties and object methods.
function Scene:ellipse(options) end

--- Built-in rectangular scene object.
---@class Kinematic.Rect: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Rect, callback:fun(object:Kinematic.Rect, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [256, 256].
---@field round Kinematic.QuadProperty Corner radii in local coordinates, ordered top-left, top-right, bottom-right, bottom-left. Default: [0, 0, 0, 0].
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Rect = {}

--- Initial properties and resources for rect.
---@class Kinematic.RectOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [256, 256].
---@field round? Kinematic.Quad|{a?:number,b?:number,c?:number,d?:number} Corner radii in local coordinates, ordered top-left, top-right, bottom-right, bottom-left. Default: [0, 0, 0, 0].
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in rectangular scene object.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.RectOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Rect object Handle exposing animatable properties and object methods.
function Scene:rect(options) end

--- Built-in triangle scene object with an independently animatable apex.
---@class Kinematic.Triangle: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Triangle, callback:fun(object:Kinematic.Triangle, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Base width and height in local coordinates. Default: [256, 256].
---@field apex Kinematic.Property<number> Horizontal apex position from left (0) to right (1). Default: 0.5. Minimum: 0. Maximum: 1.
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Triangle = {}

--- Initial properties and resources for triangle.
---@class Kinematic.TriangleOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Base width and height in local coordinates. Default: [256, 256].
---@field apex? number Horizontal apex position from left (0) to right (1). Default: 0.5. Minimum: 0. Maximum: 1.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in triangle scene object with an independently animatable apex.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.TriangleOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Triangle object Handle exposing animatable properties and object methods.
function Scene:triangle(options) end

--- Built-in regular polygon scene object.
---@class Kinematic.RegularPolygon2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.RegularPolygon2D, callback:fun(object:Kinematic.RegularPolygon2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [256, 256].
---@field sides Kinematic.Property<integer> Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local RegularPolygon2D = {}

--- Initial properties and resources for regular_polygon_2d.
---@class Kinematic.RegularPolygon2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [256, 256].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in regular polygon scene object.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.RegularPolygon2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.RegularPolygon2D object Handle exposing animatable properties and object methods.
function Scene:regular_polygon_2d(options) end

--- Built-in line scene object with independently animatable arrowheads.
---@class Kinematic.Line2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Line2D, callback:fun(object:Kinematic.Line2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field from Kinematic.Vec2Property Starting point in local coordinates. Default: [-128, 0].
---@field to Kinematic.Vec2Property Ending point in local coordinates. Default: [128, 0].
---@field thickness Kinematic.Property<number> Width of the line body. Default: 8. Minimum: 0.
---@field from_arrow_size Kinematic.Property<number> Size of the arrowhead at the starting point. Default: 0. Minimum: 0.
---@field to_arrow_size Kinematic.Property<number> Size of the arrowhead at the ending point. Default: 0. Minimum: 0.
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Line2D = {}

--- Initial properties and resources for line_2d.
---@class Kinematic.Line2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field from? Kinematic.Vec2|{x?:number,y?:number} Starting point in local coordinates. Default: [-128, 0].
---@field to? Kinematic.Vec2|{x?:number,y?:number} Ending point in local coordinates. Default: [128, 0].
---@field thickness? number Width of the line body. Default: 8. Minimum: 0.
---@field from_arrow_size? number Size of the arrowhead at the starting point. Default: 0. Minimum: 0.
---@field to_arrow_size? number Size of the arrowhead at the ending point. Default: 0. Minimum: 0.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in line scene object with independently animatable arrowheads.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Line2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Line2D object Handle exposing animatable properties and object methods.
function Scene:line_2d(options) end

--- Image drawn inside a styled rectangular shape.
---@class Kinematic.Image2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Image2D, callback:fun(object:Kinematic.Image2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [256, 256].
---@field round Kinematic.QuadProperty Corner radii in local coordinates, ordered top-left, top-right, bottom-right, bottom-left. Default: [0, 0, 0, 0].
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [0, 0, 0, 0].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Image2D = {}

--- Samples a 2D or 3D image object at a local vec2 position and returns its color.
--- Requires an image source on the object.
---@param position Kinematic.Vec2
---@return Kinematic.Color
function Image2D:get_pixel_color(position) end

--- Initial properties and resources for image_2d.
---@class Kinematic.Image2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [256, 256].
---@field round? Kinematic.Quad|{a?:number,b?:number,c?:number,d?:number} Corner radii in local coordinates, ordered top-left, top-right, bottom-right, bottom-left. Default: [0, 0, 0, 0].
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [0, 0, 0, 0].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Image drawn inside a styled rectangular shape.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Image2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Image2D object Handle exposing animatable properties and object methods.
function Scene:image_2d(options) end

--- Built-in 2D code object with syntax coloring, animated edits, and emphasis.
--- 
--- ```lua
--- local code = s:code {language = "python", code = "print(42)"}
--- s:world_2d():add(code)
--- code:append("print(100)"):play(1)
--- ```
---@class Kinematic.Code: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Code, callback:fun(object:Kinematic.Code, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field code Kinematic.Property<string> Code displayed by the object, preserving indentation. Default: "".
---@field size Kinematic.Property<number> Font size in logical canvas units. Default: 36. Minimum: 0.
---@field line_spacing Kinematic.Property<number> Line spacing as a multiple of the font's natural spacing. Default: 1. Minimum: 0.1.
---@field tab_size Kinematic.Property<integer> Number of visual columns between tab stops. Default: 4. Minimum: 1. Maximum: 16.
---@field language Kinematic.Property<'lua'|'plain'|'javascript'|'typescript'|'jsx'|'tsx'|'html'|'css'|'scss'|'python'|'c'|'cpp'|'csharp'|'java'|'rust'|'go'|'php'|'ruby'|'swift'|'kotlin'|'dart'|'bash'|'powershell'|'sql'|'r'|'json'|'jsonc'|'yaml'|'toml'|'xml'|'ini'|'markdown'|'latex'|'dockerfile'|'makefile'|'cmake'|'glsl'|'hlsl'|'wgsl'> Syntax language, defaulting to plain. Web: javascript, typescript, jsx, tsx, html, css, scss. General: python, c, cpp, csharp, java, rust, go, php, ruby, swift, kotlin, dart. Scripts and data: bash, powershell, sql, r. Configuration: json, jsonc, yaml, toml, xml, ini. Documents: markdown, latex. Build: dockerfile, makefile, cmake. Shaders: glsl, hlsl, wgsl. Default: "plain".
---@field keyword_color Kinematic.ColorProperty Color of keywords, types, and language-specific declarations. Default: [0.78, 0.47, 0.87, 1].
---@field string_color Kinematic.ColorProperty Color of string literals and quoted values. Default: [0.6, 0.76, 0.47, 1].
---@field number_color Kinematic.ColorProperty Color of numeric literals. Default: [0.82, 0.6, 0.4, 1].
---@field comment_color Kinematic.ColorProperty Color of line and long bracket comments. Default: [0.45, 0.51, 0.56, 1].
---@field operator_color Kinematic.ColorProperty Color of operators and punctuation. Default: [0.34, 0.71, 0.76, 1].
---@field highlight_color Kinematic.ColorProperty Background color of highlighted code ranges. Default: [0.3, 0.55, 1, 0.3].
---@field dim_opacity Kinematic.Property<number> Opacity multiplier for text outside the selection. Default: 0.25. Minimum: 0. Maximum: 1.
---@field selection Kinematic.Property<Kinematic.CodeRange[]> Selected half-open ranges, using one-based Unicode line and column positions. Default: [] (1.00).
---@field highlights Kinematic.Property<Kinematic.CodeRange[]> Ranges marked with a colored background, independent of selection. Default: [] (1.00).
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Code = {}

--- Appends code on a new last line. Adds a line break when needed and fades in the entire addition, preserving existing tokens.
--- 
--- ```lua
--- code:append('print(value)'):play(1)
--- ```
---@param text string
---@return Kinematic.Animation
function Code:append(text) end

--- Prepends code on a new first line. Adds a line break when needed and fades in the entire addition, preserving existing tokens.
--- 
--- ```lua
--- code:prepend('local value = 42'):play(1)
--- ```
---@param text string
---@return Kinematic.Animation
function Code:prepend(text) end

--- Alias of prepend, adding code on a new first line with a fade.
--- 
--- ```lua
--- code:prepand('local value = 42'):play(1)
--- ```
---@param text string
---@return Kinematic.Animation
function Code:prepand(text) end

--- Selects characters on one line: word(line, column, length). Line and column start at 1. Omit length to select through the end of the line. The result can be passed to select, highlight, replace, or erase.
--- 
--- ```lua
--- -- Three characters starting at column 3 of line 1.
--- code:select(code:word(1, 3, 3)):play(1)
--- -- Column 3 through the end of line 1.
--- code:highlight(code:word(1, 3)):play(1)
--- ```
---@param line integer
---@param column integer
---@param length integer|nil
---@return Kinematic.CodeRange
function Code:word(line, column, length) end

--- Selects whole lines: lines(first, last). Both line numbers start at 1 and are included. Omit last to select one line. The result can be passed to select, highlight, replace, or erase.
--- 
--- ```lua
--- code:select(code:lines(1, 3)):play(1)
--- code:highlight(code:lines(2)):play(1)
--- code:select({code:find('1'), code:find_all('oi'), code:lines(1)}):play(1)
--- ```
---@param first integer
---@param last integer|nil
---@return Kinematic.CodeRange
function Code:lines(first, last) end

--- Moves matching characters and fades inserted or removed characters. Group matches whole words and individual symbols instead. Default: false. Preparing leaves the timeline unchanged.
--- 
--- ```lua
--- code:morph('local value = 42\nprint(value)'):play(1)
--- ```
---@param text string
---@param group boolean?
---@return Kinematic.Animation
function Code:morph(text, group) end

--- Crossfades the current code into a new block.
--- 
--- ```lua
--- code:fade('print(42)'):play(1)
--- ```
---@param text string
---@return Kinematic.Animation
function Code:fade(text) end

--- Inserts text at a one-based {line, column} position. Affected tokens fade as whole words; intact tokens keep their identity. Use morph for letter matching. Columns count Unicode characters, including each tab as one character. The position is checked against the code at each play.
--- 
--- ```lua
--- -- Inserts before the first character of the second line.
--- code:insert({2, 1}, '-- Result.\n'):play(1)
--- ```
---@param position Kinematic.CodePosition
---@param text string
---@return Kinematic.Animation
function Code:insert(position, text) end

--- Replaces one or several text intervals with the same text. Use find, find_all, word, or lines to choose the intervals; their results can be mixed in a list. Changed tokens fade as whole words. Edits use the code at play time. Overlapping intervals or a missing single search result raise an error.
--- 
--- ```lua
--- -- Replaces the first occurrence.
--- code:replace(code:find('value'), 'answer'):play(1)
--- -- Replaces every occurrence of either name.
--- code:replace({code:find_all('foo'), code:find_all('bar')}, 'value'):play(1)
--- ```
---@param ranges Kinematic.CodeRanges|nil
---@param text string
---@return Kinematic.Animation
function Code:replace(ranges, text) end

--- Erases the text chosen by find, find_all, word, or lines. Accepts one interval or a list mixing their results. Changed tokens fade as whole words. Overlapping intervals or a missing single search result raise an error. Use remove to remove the scene object.
--- 
--- ```lua
--- -- Removes the first line.
--- code:erase(code:lines(1)):play(1)
--- -- Removes every occurrence.
--- code:erase(code:find_all('debug')):play(1)
--- ```
---@param ranges Kinematic.CodeRanges|nil
---@return Kinematic.Animation
function Code:erase(ranges) end

--- Dims the text outside the chosen intervals, preserving syntax colors. Choose text with find, find_all, word, or lines, and optionally mix their results in a list. Nil or {} restores all text. Missing searches inside a list are ignored. Intervals keep their coordinates after edits.
--- 
--- ```lua
--- code:select(code:find_all('value')):play(1)
--- -- Combines searches and a complete line.
--- code:select({code:find('1'), code:find_all('oi'), code:lines(1)}):play(1)
--- code:select(nil):play(1)
--- ```
---@param ranges Kinematic.CodeRanges|nil
---@return Kinematic.Animation
function Code:select(ranges) end

--- Marks the chosen text with highlight_color. Choose text with find, find_all, word, or lines, and optionally mix their results in a list. Nil or {} removes the backgrounds. Missing searches inside a list are ignored. Included in exports. Intervals keep their coordinates after edits.
--- 
--- ```lua
--- code:highlight(code:find_all('value')):play(1)
--- code:to {highlight_color = k.color('#FFCC0066')}:play(1)
--- code:highlight({}):play(1)
--- ```
---@param ranges Kinematic.CodeRanges|nil
---@return Kinematic.Animation
function Code:highlight(ranges) end

--- Finds the first literal occurrence in the code at the current scene time. Match_case and match_whole_word default to false. Whole words use letters, digits, and underscores; adjacent punctuation is allowed. Returns a CodeRange to pass to select, highlight, replace, or erase, or nil when absent. The result is a fixed range, not a live query.
--- 
--- ```lua
--- local range = code:find('value')
--- if range then
--- code:replace(range, 'answer'):play(1)
--- end
--- ```
---@param text string
---@param match_case boolean?
---@param match_whole_word boolean?
---@return Kinematic.CodeRange?
function Code:find(text, match_case, match_whole_word) end

--- Finds all non-overlapping literal occurrences at the current scene time. Match_case and match_whole_word default to false. Whole words use letters, digits, and underscores; adjacent punctuation is allowed. Returns an empty list when absent. Returned coordinates do not follow later edits.
--- 
--- ```lua
--- code:highlight(code:find_all('value')):play(1)
--- ```
---@param text string
---@param match_case boolean?
---@param match_whole_word boolean?
---@return Kinematic.CodeRange[]
function Code:find_all(text, match_case, match_whole_word) end

--- Initial properties and resources for code.
---@class Kinematic.CodeOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field code? string Code displayed by the object, preserving indentation. Default: "".
---@field size? number Font size in logical canvas units. Default: 36. Minimum: 0.
---@field line_spacing? number Line spacing as a multiple of the font's natural spacing. Default: 1. Minimum: 0.1.
---@field tab_size? integer Number of visual columns between tab stops. Default: 4. Minimum: 1. Maximum: 16.
---@field language? 'lua'|'plain'|'javascript'|'typescript'|'jsx'|'tsx'|'html'|'css'|'scss'|'python'|'c'|'cpp'|'csharp'|'java'|'rust'|'go'|'php'|'ruby'|'swift'|'kotlin'|'dart'|'bash'|'powershell'|'sql'|'r'|'json'|'jsonc'|'yaml'|'toml'|'xml'|'ini'|'markdown'|'latex'|'dockerfile'|'makefile'|'cmake'|'glsl'|'hlsl'|'wgsl' Syntax language, defaulting to plain. Web: javascript, typescript, jsx, tsx, html, css, scss. General: python, c, cpp, csharp, java, rust, go, php, ruby, swift, kotlin, dart. Scripts and data: bash, powershell, sql, r. Configuration: json, jsonc, yaml, toml, xml, ini. Documents: markdown, latex. Build: dockerfile, makefile, cmake. Shaders: glsl, hlsl, wgsl. Default: "plain".
---@field keyword_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of keywords, types, and language-specific declarations. Default: [0.78, 0.47, 0.87, 1].
---@field string_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of string literals and quoted values. Default: [0.6, 0.76, 0.47, 1].
---@field number_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of numeric literals. Default: [0.82, 0.6, 0.4, 1].
---@field comment_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of line and long bracket comments. Default: [0.45, 0.51, 0.56, 1].
---@field operator_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of operators and punctuation. Default: [0.34, 0.71, 0.76, 1].
---@field highlight_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Background color of highlighted code ranges. Default: [0.3, 0.55, 1, 0.3].
---@field dim_opacity? number Opacity multiplier for text outside the selection. Default: 0.25. Minimum: 0. Maximum: 1.
---@field selection? Kinematic.CodeRange[] Selected half-open ranges, using one-based Unicode line and column positions. Default: [] (1.00).
---@field highlights? Kinematic.CodeRange[] Ranges marked with a colored background, independent of selection. Default: [] (1.00).
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in 2D code object with syntax coloring, animated edits, and emphasis.
--- 
--- ```lua
--- local code = s:code {language = "python", code = "print(42)"}
--- s:world_2d():add(code)
--- code:append("print(100)"):play(1)
--- ```
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.CodeOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Code object Handle exposing animatable properties and object methods.
function Scene:code(options) end

--- Built-in text scene object.
---@class Kinematic.Text2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Text2D, callback:fun(object:Kinematic.Text2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field text Kinematic.Property<string> Text displayed by the object. Default: "Text!".
---@field size Kinematic.Property<number> Font size in logical canvas units. Default: 64. Minimum: 0.
---@field align Kinematic.Property<number> Horizontal line alignment from `-1` left to `1` right. Default: 0. Minimum: -1. Maximum: 1.
---@field thickness Kinematic.Property<number> Extra glyph thickness in logical canvas units. Default: 0. Minimum: 0.
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Text2D = {}

--- Moves matching glyphs and fades added or removed glyphs.
--- Group matches whole words and individual symbols instead. Default: false.
--- Returns an animation; call `:play(duration, easing)` to schedule it.
---@param text string
---@param group boolean?
---@return Kinematic.Animation
function Text2D:morph(text, group) end

--- Crossfades the current text into the new text.
--- Returns an animation; call `:play(duration, easing)` to schedule it.
---@param text string
---@return Kinematic.Animation
function Text2D:fade(text) end

--- Initial properties and resources for text_2d.
---@class Kinematic.Text2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field text? string Text displayed by the object. Default: "Text!".
---@field size? number Font size in logical canvas units. Default: 64. Minimum: 0.
---@field align? number Horizontal line alignment from `-1` left to `1` right. Default: 0. Minimum: -1. Maximum: 1.
---@field thickness? number Extra glyph thickness in logical canvas units. Default: 0. Minimum: 0.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in text scene object.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Text2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Text2D object Handle exposing animatable properties and object methods.
function Scene:text_2d(options) end

--- Built-in LaTeX math scene object, rendered with embedded KaTeX fonts.
--- 
--- Supports mathematical LaTeX accepted by RaTeX in display style.
--- Invalid or unsupported source panics when its geometry is first requested.
---@class Kinematic.Latex2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Latex2D, callback:fun(object:Kinematic.Latex2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field text Kinematic.Property<string> LaTeX math source, without dollar delimiters. Default: "e^{i\\pi}+1=0".
---@field size Kinematic.Property<number> Font size in logical canvas units. Default: 64. Minimum: 0.
---@field thickness Kinematic.Property<number> Extra glyph thickness in logical canvas units. Default: 0. Minimum: 0.
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Latex2D = {}

--- Moves matching glyphs and fades added or removed glyphs.
--- Group matches whole words and individual symbols instead. Default: false.
--- Returns an animation; call `:play(duration, easing)` to schedule it.
---@param text string
---@param group boolean?
---@return Kinematic.Animation
function Latex2D:morph(text, group) end

--- Crossfades the current formula into the new formula.
--- Returns an animation; call `:play(duration, easing)` to schedule it.
---@param text string
---@return Kinematic.Animation
function Latex2D:fade(text) end

--- Initial properties and resources for latex_2d.
---@class Kinematic.Latex2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field text? string LaTeX math source, without dollar delimiters. Default: "e^{i\\pi}+1=0".
---@field size? number Font size in logical canvas units. Default: 64. Minimum: 0.
---@field thickness? number Extra glyph thickness in logical canvas units. Default: 0. Minimum: 0.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in LaTeX math scene object, rendered with embedded KaTeX fonts.
--- 
--- Supports mathematical LaTeX accepted by RaTeX in display style.
--- Invalid or unsupported source panics when its geometry is first requested.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Latex2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Latex2D object Handle exposing animatable properties and object methods.
function Scene:latex_2d(options) end

--- Transformable scene object that groups an ordered set of child objects.
---@class Kinematic.Group2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Group2D, callback:fun(object:Kinematic.Group2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Group2D = {}

--- Initial properties and resources for group_2d.
---@class Kinematic.Group2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Transformable scene object that groups an ordered set of child objects.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Group2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Group2D object Handle exposing animatable properties and object methods.
function Scene:group_2d(options) end

--- A two-dimensional node that lays out its children procedurally.
---@class Kinematic.Container2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Container2D, callback:fun(object:Kinematic.Container2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field gap Kinematic.Vec2Property Spacing between children on each layout axis in local coordinates. Default: [0, 0].
---@field direction Kinematic.Property<'horizontal'|'horizontal_reverse'|'vertical'|'vertical_reverse'> Axis and direction used to arrange the container children. Default: "horizontal".
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Container2D = {}

--- Initial properties and resources for container_2d.
---@class Kinematic.Container2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field gap? Kinematic.Vec2|{x?:number,y?:number} Spacing between children on each layout axis in local coordinates. Default: [0, 0].
---@field direction? 'horizontal'|'horizontal_reverse'|'vertical'|'vertical_reverse' Axis and direction used to arrange the container children. Default: "horizontal".
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- A two-dimensional node that lays out its children procedurally.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Container2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Container2D object Handle exposing animatable properties and object methods.
function Scene:container_2d(options) end

--- Independent 2D Skia viewport with its own camera.
---@class Kinematic.Canvas2D: Kinematic.Object
---@field signal fun(self:Kinematic.Canvas2D, callback:fun(object:Kinematic.Canvas2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field clear Kinematic.ColorProperty Color used to clear the canvas before drawing its children. Default: [0, 0, 0, 0].
---@field filter Kinematic.Property<'nearest'|'linear'> Sampling used by previews and projections of this canvas. Defaults to "nearest". Default: "nearest".
---@field camera_position Kinematic.Vec2Property Position observed at the center of the viewport. Default: [0, 0].
---@field camera_zoom Kinematic.Property<number> Magnification applied to the scene. Default: 1. Minimum: 0.000001.
---@field camera_rotation Kinematic.Property<number> Rotation of the view in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
local Canvas2D = {}

--- Returns the live texture produced by a canvas object.
--- Use it as another object source or shader texture input.
---@return Kinematic.Texture
function Canvas2D:texture() end

--- Initial properties and resources for canvas_2d.
---@class Kinematic.Canvas2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field clear? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to clear the canvas before drawing its children. Default: [0, 0, 0, 0].
---@field filter? 'nearest'|'linear' Sampling used by previews and projections of this canvas. Defaults to "nearest". Default: "nearest".
---@field camera_position? Kinematic.Vec2|{x?:number,y?:number} Position observed at the center of the viewport. Default: [0, 0].
---@field camera_zoom? number Magnification applied to the scene. Default: 1. Minimum: 0.000001.
---@field camera_rotation? number Rotation of the view in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.

--- Independent 2D Skia viewport with its own camera.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Canvas2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Canvas2D object Handle exposing animatable properties and object methods.
function Scene:canvas_2d(options) end

--- 2D rectangle sampling the premultiplied output of a canvas.
---@class Kinematic.Projection2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Projection2D, callback:fun(object:Kinematic.Projection2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [256, 256].
---@field round Kinematic.QuadProperty Corner radii in local coordinates, ordered top-left, top-right, bottom-right, bottom-left. Default: [0, 0, 0, 0].
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [0, 0, 0, 0].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
local Projection2D = {}

--- Initial properties and resources for projection_2d.
---@class Kinematic.Projection2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [256, 256].
---@field round? Kinematic.Quad|{a?:number,b?:number,c?:number,d?:number} Corner radii in local coordinates, ordered top-left, top-right, bottom-right, bottom-left. Default: [0, 0, 0, 0].
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [0, 0, 0, 0].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- 2D rectangle sampling the premultiplied output of a canvas.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Projection2DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Projection2D object Handle exposing animatable properties and object methods.
function Scene:projection_2d(options) end

--- Sphere centered on its local origin.
---@class Kinematic.Sphere: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Sphere, callback:fun(object:Kinematic.Sphere, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field radius Kinematic.Property<number> Radius in local coordinates. Default: 0.5. Minimum: 0.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Sphere = {}

--- Initial properties and resources for sphere.
---@class Kinematic.SphereOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field radius? number Radius in local coordinates. Default: 0.5. Minimum: 0.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Sphere centered on its local origin.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.SphereOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Sphere object Handle exposing animatable properties and object methods.
function Scene:sphere(options) end

--- Torus centered on its local origin with its ring around the Y axis.
---@class Kinematic.Torus: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Torus, callback:fun(object:Kinematic.Torus, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field major_radius Kinematic.Property<number> Distance from the torus center to the center of its tube in local units. Default: 0.375. Minimum: 0.
---@field minor_radius Kinematic.Property<number> Radius of the torus tube in local units. Default: 0.125. Minimum: 0.
---@field major_segments Kinematic.Property<integer> Number of subdivisions around the torus ring, between 3 and 256. Default: 32. Minimum: 3. Maximum: 256.
---@field minor_segments Kinematic.Property<integer> Number of subdivisions around the torus tube, between 3 and 256. Default: 16. Minimum: 3. Maximum: 256.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Torus = {}

--- Initial properties and resources for torus.
---@class Kinematic.TorusOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field major_radius? number Distance from the torus center to the center of its tube in local units. Default: 0.375. Minimum: 0.
---@field minor_radius? number Radius of the torus tube in local units. Default: 0.125. Minimum: 0.
---@field major_segments? integer Number of subdivisions around the torus ring, between 3 and 256. Default: 32. Minimum: 3. Maximum: 256.
---@field minor_segments? integer Number of subdivisions around the torus tube, between 3 and 256. Default: 16. Minimum: 3. Maximum: 256.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Torus centered on its local origin with its ring around the Y axis.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.TorusOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Torus object Handle exposing animatable properties and object methods.
function Scene:torus(options) end

--- Regular-base prism centered on its local origin.
---@class Kinematic.Prism: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Prism, callback:fun(object:Kinematic.Prism, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec3Property Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides Kinematic.Property<integer> Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field top_cap Kinematic.Property<boolean> Closes the top of the prism when true. Default: true.
---@field bottom_cap Kinematic.Property<boolean> Closes the bottom of the prism when true. Default: true.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Prism = {}

--- Initial properties and resources for prism.
---@class Kinematic.PrismOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec3|{x?:number,y?:number,z?:number} Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field top_cap? boolean Closes the top of the prism when true. Default: true.
---@field bottom_cap? boolean Closes the bottom of the prism when true. Default: true.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Regular-base prism centered on its local origin.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PrismOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Prism object Handle exposing animatable properties and object methods.
function Scene:prism(options) end

--- Regular-base pyramid centered on its local origin with its apex facing positive Y.
---@class Kinematic.Pyramid: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Pyramid, callback:fun(object:Kinematic.Pyramid, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec3Property Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides Kinematic.Property<integer> Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field cap Kinematic.Property<boolean> Closes the base of the pyramid when true. Default: true.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Pyramid = {}

--- Initial properties and resources for pyramid.
---@class Kinematic.PyramidOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec3|{x?:number,y?:number,z?:number} Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field cap? boolean Closes the base of the pyramid when true. Default: true.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Regular-base pyramid centered on its local origin with its apex facing positive Y.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PyramidOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Pyramid object Handle exposing animatable properties and object methods.
function Scene:pyramid(options) end

--- Plane in local XY with its front facing positive Z.
---@class Kinematic.Plane: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Plane, callback:fun(object:Kinematic.Plane, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [1, 1].
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Plane = {}

--- Initial properties and resources for plane.
---@class Kinematic.PlaneOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [1, 1].
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Plane in local XY with its front facing positive Z.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PlaneOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Plane object Handle exposing animatable properties and object methods.
function Scene:plane(options) end

--- Flat regular polygon with its front facing positive Z.
---@class Kinematic.RegularPolygon3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.RegularPolygon3D, callback:fun(object:Kinematic.RegularPolygon3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [1, 1].
---@field sides Kinematic.Property<integer> Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local RegularPolygon3D = {}

--- Initial properties and resources for regular_polygon_3d.
---@class Kinematic.RegularPolygon3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Flat regular polygon with its front facing positive Z.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.RegularPolygon3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.RegularPolygon3D object Handle exposing animatable properties and object methods.
function Scene:regular_polygon_3d(options) end

--- Built-in 3D line scene object with independently animatable arrowheads.
---@class Kinematic.Line3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Line3D, callback:fun(object:Kinematic.Line3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field from Kinematic.Vec3Property Starting point in local coordinates. Default: [-0.5, 0, 0].
---@field to Kinematic.Vec3Property Ending point in local coordinates. Default: [0.5, 0, 0].
---@field thickness Kinematic.Property<number> Diameter of the line body. Default: 0.05. Minimum: 0.
---@field from_arrow_size Kinematic.Property<number> Length of the arrowhead at the starting point. Default: 0. Minimum: 0.
---@field to_arrow_size Kinematic.Property<number> Length of the arrowhead at the ending point. Default: 0. Minimum: 0.
---@field line_sides Kinematic.Property<integer> Number of sides around the line body. Default: 32. Minimum: 3. Maximum: 256.
---@field arrow_sides Kinematic.Property<integer> Number of sides around each arrowhead. Default: 32. Minimum: 3. Maximum: 256.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Line3D = {}

--- Initial properties and resources for line_3d.
---@class Kinematic.Line3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field from? Kinematic.Vec3|{x?:number,y?:number,z?:number} Starting point in local coordinates. Default: [-0.5, 0, 0].
---@field to? Kinematic.Vec3|{x?:number,y?:number,z?:number} Ending point in local coordinates. Default: [0.5, 0, 0].
---@field thickness? number Diameter of the line body. Default: 0.05. Minimum: 0.
---@field from_arrow_size? number Length of the arrowhead at the starting point. Default: 0. Minimum: 0.
---@field to_arrow_size? number Length of the arrowhead at the ending point. Default: 0. Minimum: 0.
---@field line_sides? integer Number of sides around the line body. Default: 32. Minimum: 3. Maximum: 256.
---@field arrow_sides? integer Number of sides around each arrowhead. Default: 32. Minimum: 3. Maximum: 256.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Built-in 3D line scene object with independently animatable arrowheads.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Line3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Line3D object Handle exposing animatable properties and object methods.
function Scene:line_3d(options) end

--- Textured plane in local XY with its front facing positive Z.
---@class Kinematic.Image3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Image3D, callback:fun(object:Kinematic.Image3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [1, 1].
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: true.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Image3D = {}

--- Samples a 2D or 3D image object at a local vec2 position and returns its color.
--- Requires an image source on the object.
---@param position Kinematic.Vec2
---@return Kinematic.Color
function Image3D:get_pixel_color(position) end

--- Initial properties and resources for image_3d.
---@class Kinematic.Image3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [1, 1].
---@field unlit? boolean Draws the surface without lighting when true. Default: true.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Textured plane in local XY with its front facing positive Z.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Image3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Image3D object Handle exposing animatable properties and object methods.
function Scene:image_3d(options) end

--- Built-in extruded text scene object.
---@class Kinematic.Text3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Text3D, callback:fun(object:Kinematic.Text3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field text Kinematic.Property<string> Text displayed by the object. Default: "Text!".
---@field size Kinematic.Property<number> Font size in world units. Default: 1. Minimum: 0.
---@field align Kinematic.Property<number> Horizontal line alignment from `-1` left to `1` right. Default: 0. Minimum: -1. Maximum: 1.
---@field thickness Kinematic.Property<number> Extra glyph thickness in world units. Default: 0. Minimum: 0.
---@field depth Kinematic.Property<number> Extrusion depth in world units. Default: 0.1. Minimum: 0.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Text3D = {}

--- Moves matching glyphs and fades added or removed glyphs.
--- Group matches whole words and individual symbols instead. Default: false.
--- Returns an animation; call `:play(duration, easing)` to schedule it.
---@param text string
---@param group boolean?
---@return Kinematic.Animation
function Text3D:morph(text, group) end

--- Initial properties and resources for text_3d.
---@class Kinematic.Text3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field text? string Text displayed by the object. Default: "Text!".
---@field size? number Font size in world units. Default: 1. Minimum: 0.
---@field align? number Horizontal line alignment from `-1` left to `1` right. Default: 0. Minimum: -1. Maximum: 1.
---@field thickness? number Extra glyph thickness in world units. Default: 0. Minimum: 0.
---@field depth? number Extrusion depth in world units. Default: 0.1. Minimum: 0.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Built-in extruded text scene object.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Text3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Text3D object Handle exposing animatable properties and object methods.
function Scene:text_3d(options) end

--- Built-in extruded LaTeX math scene object.
--- 
--- Supports mathematical LaTeX accepted by RaTeX in display style.
--- Invalid or unsupported source panics when its geometry is first requested.
---@class Kinematic.Latex3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Latex3D, callback:fun(object:Kinematic.Latex3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field text Kinematic.Property<string> LaTeX math source, without dollar delimiters. Default: "e^{i\\pi}+1=0".
---@field size Kinematic.Property<number> Formula size in world units. Default: 1. Minimum: 0.
---@field thickness Kinematic.Property<number> Extra glyph thickness in world units. Default: 0. Minimum: 0.
---@field depth Kinematic.Property<number> Extrusion depth in world units. Default: 0.1. Minimum: 0.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Latex3D = {}

--- Moves matching glyphs and fades added or removed glyphs.
--- Group matches whole words and individual symbols instead. Default: false.
--- Returns an animation; call `:play(duration, easing)` to schedule it.
---@param text string
---@param group boolean?
---@return Kinematic.Animation
function Latex3D:morph(text, group) end

--- Crossfades the current formula into the new formula.
--- Returns an animation; call `:play(duration, easing)` to schedule it.
---@param text string
---@return Kinematic.Animation
function Latex3D:fade(text) end

--- Initial properties and resources for latex_3d.
---@class Kinematic.Latex3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field text? string LaTeX math source, without dollar delimiters. Default: "e^{i\\pi}+1=0".
---@field size? number Formula size in world units. Default: 1. Minimum: 0.
---@field thickness? number Extra glyph thickness in world units. Default: 0. Minimum: 0.
---@field depth? number Extrusion depth in world units. Default: 0.1. Minimum: 0.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Built-in extruded LaTeX math scene object.
--- 
--- Supports mathematical LaTeX accepted by RaTeX in display style.
--- Invalid or unsupported source panics when its geometry is first requested.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Latex3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Latex3D object Handle exposing animatable properties and object methods.
function Scene:latex_3d(options) end

--- Container inheriting three-dimensional transforms.
---@class Kinematic.Group3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Group3D, callback:fun(object:Kinematic.Group3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Group3D = {}

--- Initial properties and resources for group_3d.
---@class Kinematic.Group3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Container inheriting three-dimensional transforms.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Group3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Group3D object Handle exposing animatable properties and object methods.
function Scene:group_3d(options) end

--- A three-dimensional node that lays out its children procedurally.
---@class Kinematic.Container3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Container3D, callback:fun(object:Kinematic.Container3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field gap Kinematic.Vec3Property Spacing between children on each layout axis in local coordinates. Default: [0, 0, 0].
---@field direction Kinematic.Property<'horizontal'|'horizontal_reverse'|'vertical'|'vertical_reverse'|'depth'|'depth_reverse'> Axis and direction used to arrange the container children. Default: "horizontal".
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Container3D = {}

--- Initial properties and resources for container_3d.
---@class Kinematic.Container3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field gap? Kinematic.Vec3|{x?:number,y?:number,z?:number} Spacing between children on each layout axis in local coordinates. Default: [0, 0, 0].
---@field direction? 'horizontal'|'horizontal_reverse'|'vertical'|'vertical_reverse'|'depth'|'depth_reverse' Axis and direction used to arrange the container children. Default: "horizontal".
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- A three-dimensional node that lays out its children procedurally.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Container3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Container3D object Handle exposing animatable properties and object methods.
function Scene:container_3d(options) end

--- Independent 3D perspective viewport with its own camera.
---@class Kinematic.Canvas3D: Kinematic.Object
---@field signal fun(self:Kinematic.Canvas3D, callback:fun(object:Kinematic.Canvas3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field clear Kinematic.ColorProperty Color used to clear the canvas before drawing its children. Default: [0, 0, 0, 0].
---@field filter Kinematic.Property<'nearest'|'linear'> Sampling used by previews and projections of this canvas. Defaults to "nearest". Default: "nearest".
---@field camera_mode Kinematic.Property<'perspective'|'orthogonal'> Projection mode. Default: "perspective".
---@field camera_position Kinematic.Vec3Property Position in canvas coordinates. Defaults to `(0, 0, 3)`. Default: [0, 0, 3].
---@field camera_rotation Kinematic.QuaternionProperty Orientation looking along local negative Z, with positive Y up. Default: [0.0, 0.0, 0.0, 1.0].
---@field camera_fov Kinematic.Property<number> Vertical field of view in radians, or view height in orthogonal mode. Default: 0.7853982. Minimum: 0.000001.
---@field camera_near Kinematic.Property<number> Near clipping plane distance. Default: 0.1. Minimum: 0.000001.
---@field camera_far Kinematic.Property<number> Far clipping plane distance. Default: 1000. Minimum: 0.000001.
---@field ambient_color Kinematic.ColorProperty Ambient light color. Alpha does not affect lighting. Default: [1, 1, 1, 1].
---@field ambient_intensity Kinematic.Property<number> Nonnegative ambient light intensity multiplier. Default: 0.4. Minimum: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Canvas3D = {}

--- Creates an animation moving a 3D canvas camera to position and aiming at target.
--- Both values must be vec3. Call :play to schedule it.
---@param options {position:Kinematic.Value,target:Kinematic.Value}
---@return Kinematic.Animation
function Canvas3D:focus_camera(options) end

--- Returns the live texture produced by a canvas object.
--- Use it as another object source or shader texture input.
---@return Kinematic.Texture
function Canvas3D:texture() end

--- Initial properties and resources for canvas_3d.
---@class Kinematic.Canvas3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field clear? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to clear the canvas before drawing its children. Default: [0, 0, 0, 0].
---@field filter? 'nearest'|'linear' Sampling used by previews and projections of this canvas. Defaults to "nearest". Default: "nearest".
---@field camera_mode? 'perspective'|'orthogonal' Projection mode. Default: "perspective".
---@field camera_position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position in canvas coordinates. Defaults to `(0, 0, 3)`. Default: [0, 0, 3].
---@field camera_rotation? Kinematic.Quaternion Orientation looking along local negative Z, with positive Y up. Default: [0.0, 0.0, 0.0, 1.0].
---@field camera_fov? number Vertical field of view in radians, or view height in orthogonal mode. Default: 0.7853982. Minimum: 0.000001.
---@field camera_near? number Near clipping plane distance. Default: 0.1. Minimum: 0.000001.
---@field camera_far? number Far clipping plane distance. Default: 1000. Minimum: 0.000001.
---@field ambient_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Ambient light color. Alpha does not affect lighting. Default: [1, 1, 1, 1].
---@field ambient_intensity? number Nonnegative ambient light intensity multiplier. Default: 0.4. Minimum: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Independent 3D perspective viewport with its own camera.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Canvas3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Canvas3D object Handle exposing animatable properties and object methods.
function Scene:canvas_3d(options) end

--- Unlit 3D plane sampling the premultiplied output of a canvas.
---@class Kinematic.Projection3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Projection3D, callback:fun(object:Kinematic.Projection3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field size Kinematic.Vec2Property Geometry dimensions in local coordinates. Default: [1, 1].
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local Projection3D = {}

--- Initial properties and resources for projection_3d.
---@class Kinematic.Projection3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [1, 1].
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Unlit 3D plane sampling the premultiplied output of a canvas.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.Projection3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.Projection3D object Handle exposing animatable properties and object methods.
function Scene:projection_3d(options) end

--- Light shining along its local negative Z axis.
---@class Kinematic.DirectionalLight3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.DirectionalLight3D, callback:fun(object:Kinematic.DirectionalLight3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field color Kinematic.ColorProperty Color emitted by the light. Default: [1, 1, 1, 1].
---@field intensity Kinematic.Property<number> Nonnegative light intensity multiplier. Default: 1. Minimum: 0.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local DirectionalLight3D = {}

--- Initial properties and resources for directional_light.
---@class Kinematic.DirectionalLight3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color emitted by the light. Default: [1, 1, 1, 1].
---@field intensity? number Nonnegative light intensity multiplier. Default: 1. Minimum: 0.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Light shining along its local negative Z axis.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.DirectionalLight3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.DirectionalLight3D object Handle exposing animatable properties and object methods.
function Scene:directional_light(options) end

--- Light shining in every direction from its position.
---@class Kinematic.PointLight3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.PointLight3D, callback:fun(object:Kinematic.PointLight3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field color Kinematic.ColorProperty Color emitted by the light. Default: [1, 1, 1, 1].
---@field intensity Kinematic.Property<number> Nonnegative light intensity multiplier. Default: 1. Minimum: 0.
---@field constant Kinematic.Property<number> Constant term of light falloff: 1 / (constant + linear * distance + quadratic * distance squared). Default: 1. Minimum: 0.
---@field linear Kinematic.Property<number> Linear coefficient of light falloff with distance. Default: 0. Minimum: 0.
---@field quadratic Kinematic.Property<number> Quadratic coefficient of light falloff with distance squared. Default: 0. Minimum: 0.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local PointLight3D = {}

--- Initial properties and resources for point_light.
---@class Kinematic.PointLight3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color emitted by the light. Default: [1, 1, 1, 1].
---@field intensity? number Nonnegative light intensity multiplier. Default: 1. Minimum: 0.
---@field constant? number Constant term of light falloff: 1 / (constant + linear * distance + quadratic * distance squared). Default: 1. Minimum: 0.
---@field linear? number Linear coefficient of light falloff with distance. Default: 0. Minimum: 0.
---@field quadratic? number Quadratic coefficient of light falloff with distance squared. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Light shining in every direction from its position.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PointLight3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.PointLight3D object Handle exposing animatable properties and object methods.
function Scene:point_light(options) end

--- Light shining in a cone along its local negative Z axis.
---@class Kinematic.SpotLight3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.SpotLight3D, callback:fun(object:Kinematic.SpotLight3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field color Kinematic.ColorProperty Color emitted by the light. Default: [1, 1, 1, 1].
---@field intensity Kinematic.Property<number> Nonnegative light intensity multiplier. Default: 1. Minimum: 0.
---@field constant Kinematic.Property<number> Constant term of light falloff: 1 / (constant + linear * distance + quadratic * distance squared). Default: 1. Minimum: 0.
---@field linear Kinematic.Property<number> Linear coefficient of light falloff with distance. Default: 0. Minimum: 0.
---@field quadratic Kinematic.Property<number> Quadratic coefficient of light falloff with distance squared. Default: 0. Minimum: 0.
---@field cutoff Kinematic.Property<number> Spotlight cone cutoff angle in radians. Default: 0.7853982. Minimum: 0.001. Maximum: 3.14159.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
local SpotLight3D = {}

--- Initial properties and resources for spot_light.
---@class Kinematic.SpotLight3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color emitted by the light. Default: [1, 1, 1, 1].
---@field intensity? number Nonnegative light intensity multiplier. Default: 1. Minimum: 0.
---@field constant? number Constant term of light falloff: 1 / (constant + linear * distance + quadratic * distance squared). Default: 1. Minimum: 0.
---@field linear? number Linear coefficient of light falloff with distance. Default: 0. Minimum: 0.
---@field quadratic? number Quadratic coefficient of light falloff with distance squared. Default: 0. Minimum: 0.
---@field cutoff? number Spotlight cone cutoff angle in radians. Default: 0.7853982. Minimum: 0.001. Maximum: 3.14159.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Light shining in a cone along its local negative Z axis.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.SpotLight3DOptions Initial properties, name, identifier, and resources.
---@return Kinematic.SpotLight3D object Handle exposing animatable properties and object methods.
function Scene:spot_light(options) end

--- Initial properties and resources for square.
---@class Kinematic.RegularPolygon2DOptions_square: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [256, 256].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].

--- Built-in regular polygon scene object.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.RegularPolygon2DOptions_square Initial properties, name, identifier, and resources.
---@return Kinematic.RegularPolygon2D object Handle exposing animatable properties and object methods.
function Scene:square(options) end

--- Initial properties and resources for triangle_3d.
---@class Kinematic.RegularPolygon3DOptions_triangle_3d: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 3. Minimum: 3. Maximum: 256.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Flat regular polygon with its front facing positive Z.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.RegularPolygon3DOptions_triangle_3d Initial properties, name, identifier, and resources.
---@return Kinematic.RegularPolygon3D object Handle exposing animatable properties and object methods.
function Scene:triangle_3d(options) end

--- Initial properties and resources for square_3d.
---@class Kinematic.RegularPolygon3DOptions_square_3d: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Flat regular polygon with its front facing positive Z.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.RegularPolygon3DOptions_square_3d Initial properties, name, identifier, and resources.
---@return Kinematic.RegularPolygon3D object Handle exposing animatable properties and object methods.
function Scene:square_3d(options) end

--- Initial properties and resources for circle_3d.
---@class Kinematic.RegularPolygon3DOptions_circle_3d: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec2|{x?:number,y?:number} Geometry dimensions in local coordinates. Default: [1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 32. Minimum: 3. Maximum: 256.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Flat regular polygon with its front facing positive Z.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.RegularPolygon3DOptions_circle_3d Initial properties, name, identifier, and resources.
---@return Kinematic.RegularPolygon3D object Handle exposing animatable properties and object methods.
function Scene:circle_3d(options) end

--- Initial properties and resources for tetrahedron.
---@class Kinematic.PrismOptions_tetrahedron: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec3|{x?:number,y?:number,z?:number} Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 3. Minimum: 3. Maximum: 256.
---@field top_cap? boolean Closes the top of the prism when true. Default: true.
---@field bottom_cap? boolean Closes the bottom of the prism when true. Default: true.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Regular-base prism centered on its local origin.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PrismOptions_tetrahedron Initial properties, name, identifier, and resources.
---@return Kinematic.Prism object Handle exposing animatable properties and object methods.
function Scene:tetrahedron(options) end

--- Initial properties and resources for cube.
---@class Kinematic.PrismOptions_cube: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec3|{x?:number,y?:number,z?:number} Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 4. Minimum: 3. Maximum: 256.
---@field top_cap? boolean Closes the top of the prism when true. Default: true.
---@field bottom_cap? boolean Closes the bottom of the prism when true. Default: true.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Regular-base prism centered on its local origin.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PrismOptions_cube Initial properties, name, identifier, and resources.
---@return Kinematic.Prism object Handle exposing animatable properties and object methods.
function Scene:cube(options) end

--- Initial properties and resources for cylinder.
---@class Kinematic.PrismOptions_cylinder: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec3|{x?:number,y?:number,z?:number} Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 32. Minimum: 3. Maximum: 256.
---@field top_cap? boolean Closes the top of the prism when true. Default: true.
---@field bottom_cap? boolean Closes the bottom of the prism when true. Default: true.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Regular-base prism centered on its local origin.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PrismOptions_cylinder Initial properties, name, identifier, and resources.
---@return Kinematic.Prism object Handle exposing animatable properties and object methods.
function Scene:cylinder(options) end

--- Initial properties and resources for cone.
---@class Kinematic.PyramidOptions_cone: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field size? Kinematic.Vec3|{x?:number,y?:number,z?:number} Geometry dimensions in local coordinates. Default: [1, 1, 1].
---@field sides? integer Number of sides of the regular polygon, between 3 and 256. Default: 32. Minimum: 3. Maximum: 256.
---@field cap? boolean Closes the base of the pyramid when true. Default: true.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.

--- Regular-base pyramid centered on its local origin with its apex facing positive Y.
--- Creates the object with the supplied initial properties at the current schedule time.
---@param options Kinematic.PyramidOptions_cone Initial properties, name, identifier, and resources.
---@return Kinematic.Pyramid object Handle exposing animatable properties and object methods.
function Scene:cone(options) end

--- Scripted 2D object with local drawing, tracked properties, and fixed-step simulation.
---@class Kinematic.Script2D: Kinematic.Spatial2D
---@field signal fun(self:Kinematic.Script2D, callback:fun(object:Kinematic.Script2D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field position Kinematic.Vec2Property Position of the entity. Default: [0, 0].
---@field origin Kinematic.Vec2Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale Kinematic.Vec2Property Scale of the entity on each axis. Default: [1, 1].
---@field rotation Kinematic.Property<number> Rotation of the entity in radians. Default: 0.
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity Kinematic.Property<number> Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index Kinematic.Property<integer> Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera Kinematic.Property<boolean> Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field fill Kinematic.ColorProperty Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke Kinematic.ColorProperty Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width Kinematic.Property<number> Width used to outline the style. Default: 0. Minimum: 0.
---@field blur Kinematic.Property<number> Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset Kinematic.Vec2Property Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur Kinematic.Property<number> Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color Kinematic.ColorProperty Shadow color. Default: [0, 0, 0, 0].
---@field auto_update Kinematic.Property<boolean> Whether [`SimulationState::on_update`] runs automatically on project frames. Drawing is unaffected, so a paused simulation keeps showing its latest state. During seek replay, the value is sampled from its historical track at every frame. Default: true.
local Script2D = {}

--- Sets simulation dependencies from own property names, property handles, or other simulation objects.
--- Dependencies are sampled at each fixed step. Cyclic dependencies are rejected.
---@param inputs table<string,Kinematic.Property|Kinematic.Object|Kinematic.Audio|string>
function Script2D:inputs(inputs) end

--- Returns an independent copy of the simulation state for inspection.
---@return table
function Script2D:state() end

--- Schedules one explicit simulation update at the current time.
--- Use auto_update to control automatic fixed-step updates.
function Script2D:update() end

--- Schedules a simulation checkpoint at the current time for later seek replay.
function Script2D:checkpoint() end

--- Schedules a callback that changes simulation state at the current time.
--- Mutable data must live in state; callbacks cannot capture mutable external upvalues.
---@param callback fun(state:table)
function Script2D:mutate(callback) end

--- Initial properties and resources for script_2d.
---@class Kinematic.Script2DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field position? Kinematic.Vec2|{x?:number,y?:number} Position of the entity. Default: [0, 0].
---@field origin? Kinematic.Vec2|{x?:number,y?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0].
---@field scale? Kinematic.Vec2|{x?:number,y?:number} Scale of the entity on each axis. Default: [1, 1].
---@field rotation? number Rotation of the entity in radians. Default: 0.
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field opacity? number Transparency applied while drawing this entity, from `0` to `1`. Default: 1. Minimum: 0. Maximum: 1.
---@field z_index? integer Stacking order among sibling objects. Higher values are drawn in front. Default: 0.
---@field follows_camera? boolean Whether this entity and its subtree are affected by the canvas camera. Default: true.
---@field fill? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to fill the style. Default: [1, 1, 1, 1].
---@field stroke? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color used to outline the style. Default: [1, 1, 1, 1].
---@field stroke_width? number Width used to outline the style. Default: 0. Minimum: 0.
---@field blur? number Gaussian blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_offset? Kinematic.Vec2|{x?:number,y?:number} Shadow offset in local canvas units. Default: [0, 0].
---@field shadow_blur? number Shadow blur sigma in local canvas units. Default: 0. Minimum: 0.
---@field shadow_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Shadow color. Default: [0, 0, 0, 0].
---@field auto_update? boolean Whether [`SimulationState::on_update`] runs automatically on project frames. Drawing is unaffected, so a paused simulation keeps showing its latest state. During seek replay, the value is sampled from its historical track at every frame. Default: true.

--- Scripted 3D object with mesh drawing, tracked properties, and fixed-step simulation.
---@class Kinematic.Script3D: Kinematic.Spatial3D
---@field signal fun(self:Kinematic.Script3D, callback:fun(object:Kinematic.Script3D, frame:Kinematic.Frame)):Kinematic.Signal Runs a callback after scene evaluation at each frame; returns a signal that can be stopped.
---@field position Kinematic.Vec3Property Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin Kinematic.Vec3Property Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation Kinematic.QuaternionProperty Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale Kinematic.Vec3Property Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility Kinematic.Property<boolean> Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field unlit Kinematic.Property<boolean> Draws the surface without lighting when true. Default: false.
---@field albedo Kinematic.ColorProperty Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic Kinematic.Property<number> Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness Kinematic.Property<number> Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color Kinematic.ColorProperty Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width Kinematic.Property<number> Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity Kinematic.Property<number> Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field auto_update Kinematic.Property<boolean> Whether [`SimulationState::on_update`] runs automatically on project frames. Drawing is unaffected, so a paused simulation keeps showing its latest state. During seek replay, the value is sampled from its historical track at every frame. Default: true.
local Script3D = {}

--- Sets simulation dependencies from own property names, property handles, or other simulation objects.
--- Dependencies are sampled at each fixed step. Cyclic dependencies are rejected.
---@param inputs table<string,Kinematic.Property|Kinematic.Object|Kinematic.Audio|string>
function Script3D:inputs(inputs) end

--- Returns an independent copy of the simulation state for inspection.
---@return table
function Script3D:state() end

--- Schedules one explicit simulation update at the current time.
--- Use auto_update to control automatic fixed-step updates.
function Script3D:update() end

--- Schedules a simulation checkpoint at the current time for later seek replay.
function Script3D:checkpoint() end

--- Schedules a callback that changes simulation state at the current time.
--- Mutable data must live in state; callbacks cannot capture mutable external upvalues.
---@param callback fun(state:table)
function Script3D:mutate(callback) end

--- Initial properties and resources for script_3d.
---@class Kinematic.Script3DOptions: Kinematic.ResourceOptions
---@field id? string Stable identifier used to preserve object identity across reloads.
---@field name? string Display name in the scene tree.
---@field position? Kinematic.Vec3|{x?:number,y?:number,z?:number} Position relative to the parent in local coordinates. Default: [0, 0, 0].
---@field origin? Kinematic.Vec3|{x?:number,y?:number,z?:number} Pivot relative to half the entity size. Values are not clamped. Default: [0, 0, 0].
---@field rotation? Kinematic.Quaternion Local orientation as a quaternion. Use k.axis_angle to construct a rotation. Default: [0.0, 0.0, 0.0, 1.0].
---@field scale? Kinematic.Vec3|{x?:number,y?:number,z?:number} Local scale multiplier on each axis. Default: [1, 1, 1].
---@field visibility? boolean Whether this entity and, for containers, its subtree are drawn. Default: true.
---@field unlit? boolean Draws the surface without lighting when true. Default: false.
---@field albedo? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Base surface color used by lighting. Default: [1, 1, 1, 1].
---@field metallic? number Metallic fraction, from zero for a dielectric to one for a metal. Default: 0. Minimum: 0. Maximum: 1.
---@field roughness? number Surface roughness, from 0.04 for a smooth surface to one for a rough surface. Default: 0.7. Minimum: 0.04. Maximum: 1.
---@field outline_color? Kinematic.Color|{r?:number,g?:number,b?:number,a?:number} Color of the mesh outline. Default: [1, 1, 1, 1].
---@field outline_width? number Width of the mesh outline in screen pixels. Default: 0. Minimum: 0.
---@field opacity? number Surface opacity, from zero for transparent to one for opaque. Default: 1. Minimum: 0. Maximum: 1.
---@field auto_update? boolean Whether [`SimulationState::on_update`] runs automatically on project frames. Drawing is unaffected, so a paused simulation keeps showing its latest state. During seek replay, the value is sampled from its historical track at every frame. Default: true.

return k
