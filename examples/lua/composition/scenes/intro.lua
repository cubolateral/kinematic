return function()
    local world = k.world_2d()
    world:add(k.panel{size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = k.group_2d{id = "content"}
    world:add(content)

    local stage = k.canvas_3d{
        id = "stage",
        resolution = {900, 450},
        camera_position = k.vec3(0, 0, 7),
    }
    k.root():add(stage)

    local screen = k.projection_2d{
        id = "screen",
        source = stage:texture(),
        size = k.vec2(900, 450),
    }
    content:add(screen)

    local sphere = k.sphere{
        id = "sphere",
        radius = 1,
        position = k.vec3(-1.4, 0, 0),
        albedo = k.color("#34D399"),
        unlit = true,
    }
    local cube = k.cube{
        id = "cube",
        position = k.vec3(1.4, 0, 0),
        albedo = k.color("#A78BFA"),
        unlit = true,
    }
    stage:add(sphere)
    stage:add(cube)
    k.all(function()
        cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
        sphere:to {position = {y = 0.7}}:play(1.5)
    end)
    sphere:to {position = {y = 0}}:play(0.8)
    k.wait(0.7)

    k.wait(0.6)
    content:to {opacity = 0}:play(0.5)
    k.wait(0.2)
end
