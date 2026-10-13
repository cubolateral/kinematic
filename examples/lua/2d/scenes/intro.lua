return function(s)
    local ball = s:circle {
        id = "ball",
        radius = 256,
        fill = k.color("#FF0000"),
        position = k.vec2(-100, 0),
    }

    s:world_2d():add(ball)
    s:all(function()
        ball:to {position = k.vec2(300, 0)}:play(2, k.ease.cubic_out)
        ball:to {radius = 96}:play(1)
    end)

    s:event("Presentation")

    local center = ball.position:get()
    local motion = ball:signal(function(obj, frame)
        obj.position:set(center + k.vec2(0, math.sin(frame.time * 3) * 40))
    end)
    s:wait(3)
    motion:stop()
    ball:to {opacity = 0}:play(0.5)
    ball:remove()
end
