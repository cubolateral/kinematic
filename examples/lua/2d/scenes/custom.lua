local path = k.path()

path:move_to(20, -40)

path:cubic_to(140, -40, 20, 40, 140, 40)

path:line_to(20, 40)

path:close()

local ribbon = k.object {
    bounds = function()
        return {x = 20, y = -40, width = 120, height = 80}
    end,
    draw = function(ctx, properties)
        ctx:fill(path, properties.fill)
    end,
}

return function(s)
    local shape = s:object(ribbon, { id = "ribbon", fill = k.color("#A78BFA") })

    s:world_2d():add(shape)

    shape.opacity:set(0)

    shape:to {opacity = 1}:play(0.5)

    local saved = shape:snapshot()

    shape:by {position = k.vec2(120, 0)}:play(1)

    shape:restore(saved):play(1)

    shape:to {opacity = 0}:play(0.5)
end
