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

