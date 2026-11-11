return function()
    local ball = k.circle{
        id = "ball",
        fill = k.color("#60A5FA"),
        position = k.vec2(-200, 0),
    }

    k.world_2d():add(ball)

    ball:to { position = k.vec2(200, 0) }:play(1)

    k.wait(1)
end
