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
--- Duration is in seconds and defaults to zero for an instantaneous change. Easing defaults to cubic_in_out; use k.ease for named curves.
---@param duration? number
---@param easing? string
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

--- Returns the local bounding box dimensions as a vector.
---@return Kinematic.Value
function Object:box_size() end

--- Returns the transformation matrix composed through the object hierarchy.
---@return Kinematic.Matrix
function Object:global_matrix() end

--- Returns the position composed through the object hierarchy.
---@return Kinematic.Value
function Object:global_position() end

--- Returns the scale composed through the object hierarchy.
---@return Kinematic.Value
function Object:global_scale() end

--- Returns the rotation composed through the object hierarchy.
--- The result is radians for a 2D object or a quaternion for a 3D object.
---@return Kinematic.Value|number
function Object:global_rotation() end

--- Returns opacity composed through the hierarchy. Requires a 2D object.
---@return number
function Object:global_opacity() end

--- Samples a 2D image object at a local position and returns its color.
--- Requires an image source on the object.
---@param position Kinematic.Value
---@return Kinematic.Value
function Object:get_pixel_color(position) end

--- Creates an animation moving a 3D canvas camera to position and aiming at target.
--- Both values must be vec3. Call :play to schedule it.
---@param options {position:Kinematic.Value,target:Kinematic.Value}
---@return Kinematic.Animation
function Object:focus_camera(options) end

--- Sets simulation dependencies from own property names, property handles, or other simulation objects.
--- Dependencies are sampled at each fixed step. Cyclic dependencies are rejected.
---@param inputs table<string,Kinematic.Property|Kinematic.Object|string>
function Object:inputs(inputs) end

--- Returns an independent copy of the simulation state for inspection.
---@return table
function Object:state() end

--- Schedules one explicit simulation update at the current time.
--- Use auto_update to control automatic fixed-step updates.
function Object:update() end

--- Schedules a simulation checkpoint at the current time for later seek replay.
function Object:checkpoint() end

--- Schedules a callback that changes simulation state at the current time.
--- Mutable data must live in state; callbacks cannot capture mutable external upvalues.
---@param callback fun(state:table)
function Object:mutate(callback) end

--- Returns the live texture produced by a canvas or projection object.
--- Use it as another object source or shader texture input.
---@return Kinematic.Texture
function Object:texture() end

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
---@field choices? string[] Allowed string values for a string property.
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
---@field inputs? table<string,Kinematic.Property|Kinematic.Object|string> Declared simulation dependencies. Omitted inputs use own properties; an empty table declares none.
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
---@field source? Kinematic.Image|Kinematic.Texture Image resource or live canvas texture.
---@field font? Kinematic.Font Font resource for text drawing.
---@field shader? Kinematic.Shader Image or mesh shader resource.
---@field uniforms? table<string,number|Kinematic.Value> Initial typed shader uniform values.
---@field textures? table<string,Kinematic.Texture> Named live textures used by the shader.
---@field resolution? integer[] Canvas texture width and height in pixels.
---@field pixels_per_unit? number Texture pixels per local canvas unit.
---@field segments? integer Fixed geometry subdivision count for a sphere.
---@field shader_padding? number Extra local padding around an image shader drawing.
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
