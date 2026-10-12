local k = require("kinematic")
local settings = k.component { speed = {default = 30, min = 0, max = 200} }
local counter = k.object {
    components = {settings},
    state = {distance = 0, random = k.random(42), noise = 0},
    inputs = {speed = "speed"},
    update = function(state, frame, inputs)
        state.distance = state.distance + inputs.speed * frame.dt
        state.noise = state.random:number() * 8
    end,
    bounds = function() return {-160, -40, 320, 80} end,
    draw = function(ctx, properties, state)
        ctx:circle(-140 + state.distance, state.noise, 18, properties.fill)
    end,
}
return k.scene("Counter", function(s)
    local object = s:object(counter, {id = "counter", fill = k.color("#FBBF24")})
    s:world_2d():add(object)
    s:play(object.speed:to(60, 3, k.ease.linear))
    object:checkpoint()
    s:wait(1)
end)
