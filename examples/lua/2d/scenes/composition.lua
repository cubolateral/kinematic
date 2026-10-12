local k = require("kinematic")

return k.scene("Canvas composition", function(s)
    local source = s:canvas_3d {id = "source", resolution = {320, 240}}
    s:root():add(source)
    
    local cube = s:cube {id = "cube", albedo = k.color("#008CFF"),size = k.vec3(1.0, 2.0, 1.0)}
    source:add(cube)
    local group = s:group_2d {id = "group", opacity = 1}
    s:world_2d():add(group)
    local projection = s:projection_2d {
        id = "screen", source = source:texture(), size = k.vec2(480, 360),
    }
    group:add(projection)
    local phase = s:track(0, "phase")
    
    group.rotation:signal(function(rotation)
        rotation:set(math.sin(phase:get()) * 0.1)
    end)
    s:loop(function()
        s:play(phase:from(0, math.pi * 2, 2, k.ease.linear))
    end)
    s:parallel(function()
        s:wait(0.5)
        s:play(cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2, 3, k.ease.linear))
    end)
    s:chain(function()
        s:play(group.scale:from(k.vec2(0.8, 0.8), k.vec2(1, 1), 1))
        s:wait(3)
    end)
end)
