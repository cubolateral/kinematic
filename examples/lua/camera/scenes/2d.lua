return function()
    local world = k.world_2d()
    for i = -2, 2 do
        world:add(k.circle{
            radius = 28,
            position = k.vec2(i * 150, 0),
            fill = k.color(i == 0 and "#008CFF" or "#FFFFFF"),
        })
    end

    k.wait(0.5)
    world:to {camera_zoom = 1.5, camera_position = k.vec2(150, 0), camera_rotation = 0.2}:play(2)
    k.wait(0.5)
    world:to {camera_zoom = 1, camera_position = k.vec2(0, 0), camera_rotation = 0}:play(2)
    k.wait(0.5)
end
