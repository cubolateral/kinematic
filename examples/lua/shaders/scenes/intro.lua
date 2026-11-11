return function()
    local world = k.world_2d()
    world:add(k.panel{size = k.vec2(1280, 720), fill = k.color("#000000")})

    local content = world:add(k.group_2d{id = "content"})

    local image_shader = k.image_shader {
        source = [[
        #version 330 core
        uniform sampler2D k_image;
        uniform vec2 k_resolution;
        uniform float pulse;
        out vec4 outColor;
        void main() {
            vec2 uv = gl_FragCoord.xy / k_resolution;
            vec4 base = texture(k_image, uv);
            float stripe = 0.65 + 0.35 * sin(uv.y * 32.0 + pulse);
            outColor = vec4(base.rgb * stripe, base.a);
        }
        ]],
    }
    local card = content:add(k.panel{
        id = "card",
        size = k.vec2(300, 260),
        position = k.vec2(-250, 0),
        fill = k.color("#008CFF"),
        shader = image_shader,
        uniforms = {pulse = 0},
    })

    local mesh_shader = k.mesh_shader {
        fragment = [[
        uniform float pulse;
        in vec3 k_world_position;
        out vec4 outColor;
        void main() {
            float light = 0.5 + 0.5 * sin(k_world_position.y * 4.0 + pulse);
            outColor = vec4(0.3 + light * 0.3, 0.3, 0.8, 1.0);
        }
        ]],
    }
    local stage = k.root():add(k.canvas_3d{
        id = "stage",
        resolution = {400, 400},
        camera_position = k.vec3(0, 0, 5),
    })

    local cube = stage:add(k.cube{
        id = "cube",
        shader = mesh_shader,
        uniforms = {pulse = 0},
    })

    content:add(k.projection_2d{
        source = stage:texture(),
        size = k.vec2(400, 400),
        position = k.vec2(260, 0),
    })
    k.all(function()
        card:to {uniforms = {pulse = 12}}:play(3, "linear")
        cube:to {uniforms = {pulse = 12}}:play(3, "linear")
        cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
    end)
    k.wait(0.8)

    k.wait(0.6)
    content:to {opacity = 0}:play(0.5)
    k.wait(0.2)
end
