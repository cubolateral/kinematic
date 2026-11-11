return function()
    k.view("3d")
    local world = k.world_3d()
    world.camera_position:set(k.vec3(0, 0, 7))
    for i = -1, 1 do
        world:add(k.cube{
            position = k.vec3(i * 1.8, 0, 0),
            albedo = k.color(i == 0 and "#008CFF" or "#FFFFFF"),
            unlit = true,
        })
    end

    k.wait(0.5)
    world:focus_camera {position = k.vec3(3, 2, 6), target = k.vec3(0, 0, 0)}:play(2)
    k.wait(0.5)
    world:focus_camera {position = k.vec3(0, 0, 7), target = k.vec3(0, 0, 0)}:play(2)
    k.wait(0.5)
end
