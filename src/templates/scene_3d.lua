return function()
    k.view("3d")

    local cube = k.cube{
        id = "cube",
        albedo = k.color("#60A5FA"),
    }

    k.world_3d():add(cube)

    cube:to { rotation = k.axis_angle(k.vec3(0, 1, 0), math.pi) }:play(1)

    k.wait(1)
end
