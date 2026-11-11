return function()
    local world = k.world_2d()
    world:add(k.panel{size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = k.group_2d{id = "content"}
    world:add(content)

    local rail = k.panel{size = k.vec2(720, 4), fill = k.color("#383838")}
    content:add(rail)

    local ball = k.circle{
        id = "ball",
        radius = 42,
        position = k.vec2(-360, 0),
        fill = k.color("#008CFF"),
        opacity = 0,
    }
    content:add(ball)

    ball:to {opacity = 1}:play(0.5)
    k.wait(0.3)
    ball:to {position = k.vec2(360, 0)}:play(2, "cubic_in_out")
    ball:to {fill = k.color("#34D399")}:play(0.5)
    k.wait(0.6)

    k.wait(0.6)
    content:to {opacity = 0}:play(0.5)
    k.wait(0.2)
end
