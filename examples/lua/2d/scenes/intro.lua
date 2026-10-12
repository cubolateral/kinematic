local k = require("kinematic")

return k.scene("Intro", function(s)
    local ball = s:circle {
        id = "ball",
        radius = 256,
        fill = k.color("#FF0000"),
        position = k.vec2(-100, 0),
    }

    s:world_2d():add(ball)
    s:all(function()
        s:play(ball.position:to(k.vec2(300, 0), 2, k.ease.cubic_out))
        s:play(ball.radius:to(96, 1))
    end)
    
    s:event("Presentation")

    local center = ball.position:get()
    local motion = ball:signal(function(obj, frame)
        obj.position:set(center + k.vec2(0, math.sin(frame.time * 3) * 40))
    end)
    s:wait(3)
    motion:stop()
    s:play(ball.opacity:to(0, 0.5))
    ball:remove()
end)
 