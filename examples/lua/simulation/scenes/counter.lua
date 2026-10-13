local settings = k.component { speed = {default = 30, min = 0, max = 200} }
local counter = k.object {
    name = "Counter",
    components = {settings},
    state = {distance = 0, random = k.random(42), noise = 0},
    inputs = {speed = "speed"},
    update = function(state, frame, inputs)
        state.distance = state.distance + inputs.speed * frame.dt
        state.noise = state.random:number() * 8
    end,
    bounds = function() return {x = -160, y = -40, width = 320, height = 80} end,
    draw = function(ctx, properties, state)
        ctx:circle(-140 + state.distance, state.noise, 18, properties.fill)
    end,
}
return function(s)
    local object = s:object(counter, {id = "counter", fill = k.color("#FBBF24")})
    s:world_2d():add(object)
    object:to {speed = 60}:play(3, k.ease.linear)
    object:checkpoint()
    s:wait(1)
end
