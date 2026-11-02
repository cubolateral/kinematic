return function(s)
    local world = s:world_2d()
    world:add(s:rect {size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = s:group_2d {id = "content"}
    world:add(content)

    local stage = s:canvas_3d {
        id = "stage",
        resolution = {900, 450},
        camera_position = k.vec3(0, 0, 7),
    }
    s:root():add(stage)

    local screen = s:projection_2d {
        id = "screen",
        source = stage:texture(),
        size = k.vec2(900, 450),
    }
    content:add(screen)

    local cube = s:cube {
        id = "cube",
        size = k.vec3(1.5, 1.5, 1.5),
        albedo = k.color("#008CFF"),
        roughness = 0.4,
    }
    stage:add(cube)
    stage:add(s:directional_light {rotation = k.axis_angle(k.vec3(1, 0, 0), -0.5)})

    cube:from {scale = {k.vec3(0.1, 0.1, 0.1), k.vec3(1, 1, 1)}}:play(0.6)
    cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, k.ease.linear)
    cube:to {albedo = k.color("#A78BFA"), metallic = 0.7}:play(0.8)
    s:wait(0.8)

    s:wait(0.6)
    content:to {opacity = 0}:play(0.5)
    s:wait(0.2)
end
