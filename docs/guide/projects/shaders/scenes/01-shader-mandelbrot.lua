local fractal = k.image_shader {
	source = [[
#version 330 core

uniform sampler2D k_image;
uniform vec2 k_resolution;
uniform float zoom;

out vec4 outColor;

void main() {
    vec2 uv = gl_FragCoord.xy / k_resolution;
    vec2 c = vec2(-0.7435, 0.1314) + (uv - 0.5) * vec2(3.5556, 2.0) / zoom;
    vec2 z = vec2(0.0);
    int steps = 0;

    for (int i = 0; i < 200; i++) {
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        if (dot(z, z) > 4.0) break;
        steps++;
    }

    float band = 0.5 + 0.5 * cos(float(steps) * 0.35);
    vec3 blue = vec3(0.0, 140.0, 255.0) / 255.0;
    vec3 color = steps == 200 ? vec3(0.0) : blue * band;
    float alpha = texture(k_image, uv).a;

    outColor = vec4(color * alpha, alpha);
}
]],
}

return function()
	local plane = k.world_2d():add(k.panel{
		size = k.vec2(960, 540),
		shader = fractal,
		uniforms = { zoom = 1 },
	})

	plane:uniform("zoom"):to(128):play(2, "quart_in_out")
	plane:uniform("zoom"):to(1):play(2, "quart_in_out")
end
