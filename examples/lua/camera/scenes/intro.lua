return function(s)
    s:view("3d")
    local world = s:world_3d()
    world.camera_position:set(k.vec3(0, 0, 7))
    for i = -1, 1 do
        world:add(s:cube {
            position = k.vec3(i * 1.8, 0, 0),
            albedo = k.color(i == 0 and "#008CFF" or "#FFFFFF"),
            unlit = true,
        })
    end

    s:wait(0.5)
    world:focus_camera {position = k.vec3(3, 2, 6), target = k.vec3(0, 0, 0)}:play(2)
    s:wait(0.5)
    world:focus_camera {position = k.vec3(0, 0, 7), target = k.vec3(0, 0, 0)}:play(2)
    s:wait(0.5)
end
