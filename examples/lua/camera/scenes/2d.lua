return function(s)
    local world = s:world_2d()
    for i = -2, 2 do
        world:add(s:circle {
            radius = 28,
            position = k.vec2(i * 150, 0),
            fill = k.color(i == 0 and "#008CFF" or "#FFFFFF"),
        })
    end

    s:wait(0.5)
    world:to {camera_zoom = 1.5, camera_position = k.vec2(150, 0), camera_rotation = 0.2}:play(2)
    s:wait(0.5)
    world:to {camera_zoom = 1, camera_position = k.vec2(0, 0), camera_rotation = 0}:play(2)
    s:wait(0.5)
end
