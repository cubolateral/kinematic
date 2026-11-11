local geometry = k.mesh {
    vertices = {
        {-0.6, -0.5, 0},
        {0.6, -0.5, 0},
        {0, 0.6, 0},
        {0, 0, 0.8},
    },
    indices = {1, 3, 2, 1, 2, 4, 2, 3, 4, 3, 1, 4},
}

local sculpture = k.object_3d("Sculpture", {
    on_bounds = function(self)
        return k.box(-2.6, -0.5, 0, 5.2, 1.1, 0.8)
    end,
    on_draw = function(self, ctx)
        ctx:instances(geometry, {
            k.mat4():translate(k.vec3(-2, 0, 0)),
            k.mat4(),
            k.mat4():translate(k.vec3(2, 0, 0)),
        })
    end,
})

return function()
    k.view("3d")
    k.world_3d().camera_position:set(k.vec3(0, 0, 7))
    local object = k.world_3d():add(sculpture({
        id = "sculpture",
        albedo = k.color("#008CFF"),
        unlit = true,
    }))

    object.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
    k.wait(0.5)
end
