return function()
    local world = k.world_2d()
    world:add(k.panel{size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = k.group_2d{id = "content"}
    world:add(content)

    local source = k.audio_synth {
        preset = "pluck",
        frequency = 520,
    }
    local hit = k.audio {source = source, gain = 0.3}
    local floor = k.panel{
        size = k.vec2(700, 4),
        position = k.vec2(0, 140),
        fill = k.color("#383838"),
    }
    content:add(floor)

    local ball = k.circle{
        id = "ball",
        radius = 30,
        fill = k.color("#FBBF24"),
        position = k.vec2(-250, -120),
    }
    content:add(ball)

    for i = 1, 3 do
        ball:to {position = k.vec2((i - 2) * 250, 108)}:play(0.5, "cubic_in")
        hit:play {pan = (i - 2) * 0.7}
        ball:to {position = {y = -120}}:play(0.5, "cubic_out")
    end

    local recording = k.audio {
        source = k.load_audio("assets/chime.wav"),
        gain = 0.25,
    }
    recording:play()
    k.wait(0.7)

    k.wait(0.6)
    content:to {opacity = 0}:play(0.5)
    k.wait(0.2)
end
