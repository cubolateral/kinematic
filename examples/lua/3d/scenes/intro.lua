local geometry = k.mesh {
    vertices = {{-1,-1,0}, {1,-1,0}, {0,1,0}, {0,0,1.6}},
    indices = {1,3,2, 1,2,4, 2,3,4, 3,1,4},
}
local shader = k.mesh_shader {
    fragment = [[
        uniform vec4 materialColor;
        uniform float pulse;
        in vec3 k_world_position;
        out vec4 outColor;
        void main() {
            float light = 0.65 + 0.35 * sin(k_world_position.y * 3.0 + pulse);
            outColor = vec4(materialColor.rgb * light, materialColor.a);
        }
    ]],
}
local sculpture = k.object {
    dimension = 3,
    bounds = function() return {x = -1, y = -1, z = -1.6, width = 2, height = 2, depth = 3.2} end,
    draw = function(ctx) ctx:mesh(geometry) end,
}

return function(s)
    s:view("3d")
    s:world_3d().camera_position:set(k.vec3(0, 0, 7))
    local mesh = s:object(sculpture, {
        id = "sculpture", albedo = k.color("#60A5FA"),
        shader = shader, uniforms = {pulse = 0},
        position = k.vec3(-1.5, 0, 0),
    })
    s:world_3d():add(mesh)
    local badge = s:canvas_2d {id = "badge", resolution = {128, 128}}
    s:root():add(badge)
    badge:add(s:circle {id = "badge_dot", radius = 40, fill = k.color("#FBBF24")})
    local screen = s:projection_3d {
        id = "badge_screen", source = badge:texture(),
        size = k.vec2(1, 1), position = k.vec3(0, 1.6, 0),
    }
    s:world_3d():add(screen)
    s:all(function()
        mesh:to {position = k.vec3(1.5, 0, 0)}:play(3, k.ease.sine_in_out)
        mesh:to {rotation = k.axis_angle(k.vec3(0, 1, 0), math.pi)}:play(3)
        mesh:to {uniforms = {pulse = 6}}:play(3)
    end)
    s:wait(0.5)
end
