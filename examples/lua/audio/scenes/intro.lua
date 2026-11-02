return function(s)
    local world = s:world_2d()
    world:add(s:rect {size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = s:group_2d {id = "content"}
    world:add(content)

    local source = k.audio_synth {
        preset = "pluck",
        frequency = 520,
    }
    local hit = s:audio {source = source, gain = 0.3}
    local floor = s:rect {
        size = k.vec2(700, 4),
        position = k.vec2(0, 140),
        fill = k.color("#383838"),
    }
    content:add(floor)

    local ball = s:circle {
        id = "ball",
        radius = 30,
        fill = k.color("#FBBF24"),
        position = k.vec2(-250, -120),
    }
    content:add(ball)

    for i = 1, 3 do
        ball:to {position = k.vec2((i - 2) * 250, 108)}:play(0.5, k.ease.cubic_in)
        hit:play {pan = (i - 2) * 0.7}
        ball:to {position = {y = -120}}:play(0.5, k.ease.cubic_out)
    end

    local recording = s:audio {
        source = k.audio_file("assets/chime.wav"),
        gain = 0.25,
    }
    recording:play()
    s:wait(0.7)

    s:wait(0.6)
    content:to {opacity = 0}:play(0.5)
    s:wait(0.2)
end
