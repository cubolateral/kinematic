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

    local sphere = s:sphere {
        id = "sphere",
        radius = 1,
        position = k.vec3(-1.4, 0, 0),
        albedo = k.color("#34D399"),
        unlit = true,
    }
    local cube = s:cube {
        id = "cube",
        position = k.vec3(1.4, 0, 0),
        albedo = k.color("#A78BFA"),
        unlit = true,
    }
    stage:add(sphere)
    stage:add(cube)
    s:all(function()
        cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, k.ease.linear)
        sphere:to {position = {y = 0.7}}:play(1.5)
    end)
    sphere:to {position = {y = 0}}:play(0.8)
    s:wait(0.7)

    s:wait(0.6)
    content:to {opacity = 0}:play(0.5)
    s:wait(0.2)
end
