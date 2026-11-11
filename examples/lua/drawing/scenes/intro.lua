return function()
    local world = k.world_2d()
    world:add(k.panel{size = k.vec2(1280, 720), fill = k.color("#000000")})
    local content = world:add(k.group_2d{id = "content"})

    local ribbon = k.object_2d("Ribbon", {
        on_bounds = function(self)
            return k.rect(-300, -150, 600, 300)
        end,
        on_draw = function(self, ctx)
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
    })
    local shape = content:add(ribbon({id = "ribbon"}))

    shape:from {opacity = {0, 1}, scale = {k.vec2(0.7, 0.7), k.vec2(1, 1)}}:play(1)
    local saved = shape:snapshot()
    shape:to {rotation = -0.15, position = k.vec2(0, -30)}:play(1)

    shape:restore(saved):play(1)
    k.wait(0.7)

    k.wait(0.6)
    content:to {opacity = 0}:play(0.5)
    k.wait(0.2)
end
