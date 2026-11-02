return function(s)
    local world = s:world_2d()
    local orbit = s:circle {
        radius = 170,
        fill = k.color("#00000000"),
        stroke = k.color("#383838"),
        stroke_width = 2,
    }
    local dot = s:circle {
        id = "dot",
        radius = 28,
        fill = k.color("#008CFF"),
        position = k.vec2(170, 0),
    }
    world:add(orbit)
    world:add(dot)

    local phase = s:track(0, "phase")
    local orbit_signal = dot.position:signal(function(position)
        local angle = phase:get()
        position:set(k.vec2(math.cos(angle) * 170, math.sin(angle) * 170))
    end)
    phase:to(math.pi * 2):play(3, "linear")
    orbit_signal:stop()
    s:wait(0.5)

    s:all(function()
        dot:to {opacity = 0}:play(0.5)
        orbit:to {opacity = 0}:play(0.5)
    end)
end
