return function()
    local world = k.world_2d()
    world:add(k.panel{size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = k.group_2d{id = "content"}
    world:add(content)

    local values = {42, 65, 58, 86}
    local bars = {}
    for i, value in ipairs(values) do
        local x = -330 + (i - 1) * 220
        local track = k.panel{
            size = k.vec2(110, 260),
            position = k.vec2(x, 0),
            fill = k.color("#242424"),
        }
        content:add(track)
        bars[i] = k.panel{
            id = "bar_" .. i,
            size = k.vec2(110, 0),
            position = k.vec2(x, 130),
            fill = k.color("#008CFF"),
        }
        content:add(bars[i])
    end
    k.all(function()
        for i, value in ipairs(values) do
            k.chain(function()
                k.wait((i - 1) * 0.18)
                bars[i]:to {
                    size = k.vec2(110, value * 2.6),
                    position = {y = 130 - value * 1.3},
                }:play(1.5)
            end)
        end
    end)
    bars[4]:to {fill = k.color("#34D399")}:play(0.4)
    k.wait(1)

    k.wait(0.6)
    content:to {opacity = 0}:play(0.5)
    k.wait(0.2)
end
