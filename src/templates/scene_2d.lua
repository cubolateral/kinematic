return function(s)
    local ball = s:circle {
        id = "ball",
        fill = k.color("#60A5FA"),
        position = k.vec2(-200, 0),
    }
    s:world_2d():add(ball)

    ball:to { position = k.vec2(200, 0) }:play(1)

    s:wait(1)
end
