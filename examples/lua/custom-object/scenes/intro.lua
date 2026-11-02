local progress = k.component {
    value = {default = 0, min = 0, max = 1},
    width = {default = 520, min = 1, max = 800},
    height = {default = 24, min = 1, max = 100},
}

local meter = k.object {
    name = "Meter",
    dimension = 2,
    components = {progress},
    bounds = function(properties)
        return {
            x = -properties.width / 2,
            y = -properties.height / 2,
            width = properties.width,
            height = properties.height,
        }
    end,
    draw = function(ctx, properties)
        local x, y = -properties.width / 2, -properties.height / 2
        ctx:rect(x, y, properties.width, properties.height, k.color("#242424"))
        ctx:rect(
            x,
            y,
            properties.width * properties.value,
            properties.height,
            properties.fill
        )
    end,
    methods = {
        advance = function(self, amount)
            return self:by {value = amount}
        end,
    },
}

return function(s)
    local first = s:object(meter, {
        id = "first",
        fill = k.color("#008CFF"),
        position = k.vec2(0, -60),
    })
    local second = s:object(meter, {
        id = "second",
        fill = k.color("#FFFFFF"),
        position = k.vec2(0, 60),
    })
    s:world_2d():add(first)
    s:world_2d():add(second)
    s:all(function()
        first:advance(0.8):play(2)
        second:advance(0.5):play(2)
    end)
    s:wait(0.7)

    s:all(function()
        first:reset():play(1)
        second:reset():play(1)
    end)
    s:wait(0.5)
end
