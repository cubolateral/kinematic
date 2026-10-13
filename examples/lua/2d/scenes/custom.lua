local k = require("kinematic")
local path = k.path()

path:move_to(20, -40)
path:cubic_to(140, -40, 20, 40, 140, 40)
path:line_to(20, 40)
path:close()

local ribbon = k.object {
    bounds = function() return { 20, -40, 120, 80 } end,
    draw = function(ctx, properties)
        ctx:fill(path, properties.fill)
    end,
}

return k.scene("Custom path", function(s)
    local shape = s:object(ribbon, { id = "ribbon", fill = k.color("#A78BFA") })
    s:world_2d():add(shape)
    shape.opacity:set(0)
    s:play(shape.opacity:to(1, 0.5))
    local saved = shape:snapshot()
    s:play(shape.position:by(k.vec2(120, 0), 1))
    s:play(shape:restore(saved, 1))
    s:play(shape.opacity:to(0, 0.5))
end)
