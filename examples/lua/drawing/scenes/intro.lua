return function(s)
    local world = s:world_2d()
    world:add(s:rect {size = k.vec2(1280, 720), fill = k.color("#000000")})
    local content = s:group_2d {id = "content"}
    world:add(content)

    local ribbon = k.object {
        bounds = function()
            return {x = -300, y = -150, width = 600, height = 300}
        end,
        draw = function(ctx)
            local path = k.path()
            path:move_to(-300, 100)
            path:cubic_to(-150, -220, 150, 220, 300, -100)
            path:line_to(300, 0)
            path:cubic_to(150, 300, -150, -140, -300, 150)
            path:close()
            ctx:fill(path, {
                from = {-300, 0},
                to = {300, 0},
                colors = {
                    k.color("#008CFF"),
                    k.color("#A78BFA"),
                    k.color("#FB7185"),
                },
            })
            ctx:stroke(path, k.color("#F1F5F9"), 2)
        end,
    }
    local shape = s:object(ribbon, {id = "ribbon"})
    content:add(shape)

    shape:from {opacity = {0, 1}, scale = {k.vec2(0.7, 0.7), k.vec2(1, 1)}}:play(1)
    local saved = shape:snapshot()
    shape:to {rotation = -0.15, position = k.vec2(0, -30)}:play(1)

    shape:restore(saved):play(1)
    s:wait(0.7)

    s:wait(0.6)
    content:to {opacity = 0}:play(0.5)
    s:wait(0.2)
end
