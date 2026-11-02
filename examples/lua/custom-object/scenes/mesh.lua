local geometry = k.mesh {
    vertices = {
        {-0.6, -0.5, 0},
        {0.6, -0.5, 0},
        {0, 0.6, 0},
        {0, 0, 0.8},
    },
    indices = {1, 3, 2, 1, 2, 4, 2, 3, 4, 3, 1, 4},
}

local sculpture = k.object {
    name = "Sculpture",
    dimension = 3,
    bounds = function()
        return {x = -2.6, y = -0.5, z = 0, width = 5.2, height = 1.1, depth = 0.8}
    end,
    draw = function(ctx)
        ctx:instances(geometry, {
            k.mat4():translate(k.vec3(-2, 0, 0)),
            k.mat4(),
            k.mat4():translate(k.vec3(2, 0, 0)),
        })
    end,
}

return function(s)
    s:view("3d")
    s:world_3d().camera_position:set(k.vec3(0, 0, 7))
    local object = s:object(sculpture, {
        id = "sculpture",
        albedo = k.color("#008CFF"),
        unlit = true,
    })
    s:world_3d():add(object)

    object.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
    s:wait(0.5)
end
