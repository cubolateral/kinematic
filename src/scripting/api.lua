---@meta
---@class Kinematic.Value
---@field x number
---@field y number
---@field z number
---@field w number
---@operator add(Kinematic.Value): Kinematic.Value
---@operator sub(Kinematic.Value): Kinematic.Value
---@operator mul(number): Kinematic.Value
---@class Kinematic.Task
local Task = {}
---@param seconds number
---@return Kinematic.Task
function Task:delay(seconds) end
---@class Kinematic.Property<T>
---@field get fun(self:Kinematic.Property<T>):T
---@field set fun(self:Kinematic.Property<T>, value:T)
---@field to fun(self:Kinematic.Property<T>, value:T, duration?:number, easing?:string):Kinematic.Task
---@field by fun(self:Kinematic.Property<T>, delta:T, duration?:number, easing?:string):Kinematic.Task
---@field from fun(self:Kinematic.Property<T>, start:T, finish:T, duration?:number, easing?:string):Kinematic.Task
---@field signal fun(self:Kinematic.Property<T>, callback:fun(property:Kinematic.Property<T>, frame:Kinematic.Frame)):Kinematic.Signal
---@field rotate fun(self:Kinematic.Property<T>, axis:Kinematic.Value, radians:number, duration?:number, easing?:string):Kinematic.Task
---@class Kinematic.Object
local Object = {}
---@param child Kinematic.Object
function Object:add(child) end
function Object:remove() end
---@param callback fun(object: Kinematic.Object, frame: {time:number,index:integer,dt:number})
---@return {stop:fun(self:table)}
function Object:signal(callback) end
---@class Kinematic.Scene
local Scene = {}
---@param task Kinematic.Task
function Scene:play(task) end
---@param seconds number
function Scene:wait(seconds) end
---@param name string
function Scene:event(name) end
---@param schedule fun(scene: Kinematic.Scene)
function Scene:all(schedule) end
---@param schedule fun(scene: Kinematic.Scene)
function Scene:chain(schedule) end
---@param schedule fun(scene: Kinematic.Scene)
function Scene:parallel(schedule) end
---@param schedule fun(scene: Kinematic.Scene)
function Scene:loop(schedule) end
---@return Kinematic.Canvas2D
function Scene:world_2d() end
---@return Kinematic.Canvas3D
function Scene:world_3d() end
---@return Kinematic.Object
function Scene:root() end
local k = {}
---@param name string
---@param build fun(scene: Kinematic.Scene)
---@return table
function k.scene(name, build) end
---@param x number
---@param y number
---@return Kinematic.Value
function k.vec2(x, y) end
---@param x number
---@param y number
---@param z number
---@return Kinematic.Value
function k.vec3(x, y, z) end
---@param color string|number
---@param g? number
---@param b? number
---@param a? number
---@return Kinematic.Value
function k.color(color, g, b, a) end
---@param x number
---@param y number
---@param z number
---@param w number
---@return Kinematic.Value
function k.quat(x, y, z, w) end
---@type table<string,string>
k.ease = {}
---@class Kinematic.Frame
---@field time number
---@field index integer
---@field dt number
---@class Kinematic.Signal
local Signal = {}
function Signal:stop() end
---@class Kinematic.Snapshot
---@return Kinematic.Snapshot
function Object:snapshot() end
function Object:save() end
---@param snapshot? Kinematic.Snapshot
---@param duration? number
---@param easing? string
---@return Kinematic.Task
function Object:restore(snapshot, duration, easing) end
---@param duration? number
---@param easing? string
---@return Kinematic.Task
function Object:reset(duration, easing) end
---@return Kinematic.Object[]
function Object:children() end
---@return Kinematic.Object?
function Object:parent() end
---@param type_name string
---@return Kinematic.Object?
function Object:ancestor(type_name) end
---@return string
function Object:type_name() end
---@param index integer
---@return Kinematic.Object
function Object:get_child(index) end
---@param child Kinematic.Object
---@param index integer
function Object:insert(child, index) end
function Object:remove_children() end
---@param index integer
function Object:move_to(index) end
function Object:move_up() end
function Object:move_down() end
function Object:move_to_top() end
function Object:move_to_bottom() end
---@return Kinematic.Value
function Object:bounds() end
---@return Kinematic.Matrix
function Object:global_matrix() end
---@return Kinematic.Value
function Object:global_position() end
---@return Kinematic.Value
function Object:global_scale() end
---@return Kinematic.Value|number
function Object:global_rotation() end
---@return number
function Object:global_opacity() end
---@param position Kinematic.Value
---@return Kinematic.Value
function Object:get_pixel_color(position) end
---@param position Kinematic.Value
---@param target Kinematic.Value
---@param duration? number
---@param easing? string
---@return Kinematic.Task
function Object:focus_camera(position, target, duration, easing) end
---@param inputs table<string,Kinematic.Property|Kinematic.Object|string>
function Object:inputs(inputs) end
---@return table
function Object:state() end
function Object:update() end
function Object:checkpoint() end
---@param callback fun(state:table)
function Object:mutate(callback) end
---@return Kinematic.Texture
function Object:texture() end
---@param name string
---@return Kinematic.Property
function Object:uniform(name) end
---@generic T
---@param initial T
---@param id? string
---@return Kinematic.Property<T>
function Scene:track(initial, id) end
---@param name string
---@param options table
---@return Kinematic.Object
function Scene:native(name, options) end
---@generic T
---@param definition T
---@param options table
---@return T
function Scene:object(definition, options) end
---@class Kinematic.Component
---@class Kinematic.PropertyDefinition
---@field default boolean|number|string|Kinematic.Value
---@field min? number
---@field max? number
---@field choices? string[]
---@param fields table<string,Kinematic.PropertyDefinition>
---@return Kinematic.Component
function k.component(fields) end
---@class Kinematic.ObjectDefinition
---@class Kinematic.ObjectOptions
---@field dimension? 2|3
---@field components? Kinematic.Component[]
---@field state? table
---@field inputs? table<string,Kinematic.Property|Kinematic.Object|string>
---@field update? fun(state:table, frame:Kinematic.Frame, inputs:table)
---@field draw? fun(context:Kinematic.Draw2D|Kinematic.Draw3D, properties:table, state:table)
---@field bounds? fun(properties:table, state:table):number[]
---@param definition Kinematic.ObjectOptions
---@return Kinematic.Script2D|Kinematic.Script3D
function k.object(definition) end
---@param axis Kinematic.Value
---@param radians number
---@return Kinematic.Value
function k.axis_angle(axis, radians) end
---@param a number
---@param b number
---@param c number
---@param d number
---@return Kinematic.Value
function k.quad(a, b, c, d) end
---@param x number
---@param y number
---@param width number
---@param height number
---@return Kinematic.Rect
function k.rect(x, y, width, height) end
---@class Kinematic.Rect
---@field x number
---@field y number
---@field width number
---@field height number
---@class Kinematic.Value
local Value = {}
---@param other Kinematic.Value
---@return number
function Value:dot(other) end
---@param other Kinematic.Value
---@return Kinematic.Value
function Value:cross(other) end
---@return number
function Value:length() end
---@return Kinematic.Value
function Value:normalized() end
---@param other Kinematic.Value
---@param t number
---@return Kinematic.Value
function Value:lerp(other, t) end
---@class Kinematic.Random
local Random = {}
---@param seed integer
---@return Kinematic.Random
function k.random(seed) end
---@return number
function Random:number() end
---@return integer
function Random:integer() end
---@return boolean
function Random:boolean() end
---@param key integer
---@return Kinematic.Random
function Random:fork(key) end
---@class Kinematic.Path
local Path = {}
---@return Kinematic.Path
function k.path() end
---@param x number
---@param y number
function Path:move_to(x, y) end
---@param x number
---@param y number
function Path:line_to(x, y) end
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
function Path:quad_to(x1, y1, x2, y2) end
---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param x3 number
---@param y3 number
function Path:cubic_to(x1, y1, x2, y2, x3, y3) end
function Path:close() end
---@class Kinematic.Matrix
---@operator mul(Kinematic.Matrix): Kinematic.Matrix
local Matrix = {}
---@param columns? number[]
---@return Kinematic.Matrix
function k.mat4(columns) end
---@param value Kinematic.Value
---@return Kinematic.Matrix
function Matrix:translate(value) end
---@param value Kinematic.Value
---@return Kinematic.Matrix
function Matrix:scale(value) end
---@param quaternion Kinematic.Value
---@return Kinematic.Matrix
function Matrix:rotate(quaternion) end
---@class Kinematic.Mesh
---@param data {vertices:number[][],indices:integer[],normals?:number[][],uvs?:number[][],colors?:Kinematic.Value[]}
---@return Kinematic.Mesh
function k.mesh(data) end
---@class Kinematic.Image
---@class Kinematic.Font
---@class Kinematic.Shader
---@class Kinematic.Texture
---@param path string
---@return Kinematic.Image
function k.image(path) end
---@param path string
---@return Kinematic.Font
function k.font(path) end
---@param options {source?:string,path?:string}
---@return Kinematic.Shader
function k.image_shader(options) end
---@param options {fragment?:string,fragment_path?:string,vertex?:string,vertex_path?:string}
---@return Kinematic.Shader
function k.mesh_shader(options) end
---@class Kinematic.ResourceOptions
---@field source? Kinematic.Image|Kinematic.Texture
---@field font? Kinematic.Font
---@field shader? Kinematic.Shader
---@field uniforms? table<string,number|Kinematic.Value>
---@field textures? table<string,Kinematic.Texture>
---@field resolution? integer[]
---@field pixels_per_unit? number
---@field segments? integer
---@field shader_padding? number
---@alias Kinematic.Paint Kinematic.Value|{from:number[],to:number[],colors:Kinematic.Value[]}
---@class Kinematic.Draw2D
local Draw2D = {}
function Draw2D:save() end
function Draw2D:restore() end
---@param opacity number
function Draw2D:layer(opacity) end
---@param x number
---@param y number
function Draw2D:translate(x, y) end
---@param x number
---@param y number
function Draw2D:scale(x, y) end
---@param radians number
function Draw2D:rotate(radians) end
---@param path Kinematic.Path
function Draw2D:clip(path) end
---@param path Kinematic.Path
---@param paint Kinematic.Paint
function Draw2D:fill(path, paint) end
---@param path Kinematic.Path
---@param paint Kinematic.Paint
---@param width number
function Draw2D:stroke(path, paint, width) end
---@param x number
---@param y number
---@param width number
---@param height number
---@param paint Kinematic.Paint
function Draw2D:rect(x, y, width, height, paint) end
---@param x number
---@param y number
---@param radius number
---@param paint Kinematic.Paint
function Draw2D:circle(x, y, radius, paint) end
---@param image Kinematic.Image
---@param x number
---@param y number
---@param width number
---@param height number
function Draw2D:image(image, x, y, width, height) end
---@param text string
---@param x number
---@param y number
---@param size number
---@param paint Kinematic.Paint
---@param font? Kinematic.Font
function Draw2D:text(text, x, y, size, paint, font) end
---@class Kinematic.Draw3D
local Draw3D = {}
---@param mesh Kinematic.Mesh
---@param matrix? Kinematic.Matrix
function Draw3D:mesh(mesh, matrix) end
---@param mesh Kinematic.Mesh
---@param matrices Kinematic.Matrix[]
function Draw3D:instances(mesh, matrices) end
---@param min number
---@param max number
---@return number
function Random:range(min, max) end
---@param min integer
---@param max integer
---@return integer
function Random:range_integer(min, max) end
---@param probability number
---@return boolean
function Random:chance(probability) end
---@param mean number
---@param deviation number
---@return number
function Random:normal(mean, deviation) end
---@generic T
---@param values T[]
---@param weights? number[]
---@return T?
function Random:choose(values, weights) end
---@generic T
---@param values T[]
---@param amount integer
---@return T[]
function Random:choose_multiple(values, amount) end
---@generic T
---@param values T[]
---@return T[]
function Random:shuffle(values) end
