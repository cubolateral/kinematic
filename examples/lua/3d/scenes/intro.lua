return function()
    local world = k.world_2d()
    world:add(k.panel{size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = world:add(k.group_2d{id = "content"})

    local stage = k.root():add(k.canvas_3d{
        id = "stage",
        resolution = {900, 450},
        camera_position = k.vec3(0, 0, 7),
    })

    local screen = content:add(k.projection_2d{
        id = "screen",
        source = stage:texture(),
        size = k.vec2(900, 450),
    })

    local cube = stage:add(k.cube{
        id = "cube",
        size = k.vec3(1.5, 1.5, 1.5),
        albedo = k.color("#008CFF"),
        roughness = 0.4,
    })
    stage:add(k.directional_light{rotation = k.axis_angle(k.vec3(1, 0, 0), -0.5)})

    cube:from {scale = {k.vec3(0.1, 0.1, 0.1), k.vec3(1, 1, 1)}}:play(0.6)
    cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
    cube:to {albedo = k.color("#A78BFA"), metallic = 0.7}:play(0.8)
    k.wait(0.8)

    k.wait(0.6)
    content:to {opacity = 0}:play(0.5)
    k.wait(0.2)
end
