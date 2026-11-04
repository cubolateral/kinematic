local material = k.mesh_shader {
	fragment = [[
uniform float phase;

in vec3 k_world_position;
in vec3 k_normal;
out vec4 outColor;

void main() {
    vec3 p = k_world_position;
    float wave = sin(p.y * 16.0 + 3.0 * sin(p.x * 5.0 + phase));
    float band = smoothstep(-0.2, 0.2, wave);
    vec3 blue = vec3(0.0, 140.0, 255.0) / 255.0;
    vec3 white = vec3(1.0);
    vec3 color = mix(blue, white, band);
    vec3 normal = normalize(k_normal);
    vec3 light = normalize(vec3(-1.0, 1.0, 2.0));
    float shade = 0.25 + 0.75 * max(dot(normal, light), 0.0);

    outColor = vec4(color * shade, 1.0);
}
]],
}

return function(s)
	s:view("3d")
	s:world_3d():focus_camera {
		position = k.vec3(2, 2, 2),
		target = k.vec3(0, 0, 0)
	}:play()

	local sphere = s:sphere {
		radius = 0.8,
		segments = 64,
		shader = material,
		uniforms = { phase = 0 },
	}
	s:world_3d():add(sphere)

	sphere:uniform("phase"):to(math.pi * 2):play(2, "linear")
end
