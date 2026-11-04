return function(s)
	local triangle = s:triangle {
		size = k.vec2(128, 128),
		position = k.vec2(-192, 0),
		fill = k.color("#FF4040"),
	}
	s:world_2d():add(triangle)

	local square = s:square {
		size = k.vec2(128, 128),
		fill = k.color("#40FF80"),
	}
	s:world_2d():add(square)

	local circle = s:circle {
		radius = 64,
		position = k.vec2(192, 0),
		fill = k.color("#4080FF"),
	}
	s:world_2d():add(circle)

	s:world_2d():to { camera_position = triangle.position:get(), camera_zoom = 2.5, }:play(1)
	s:world_2d():to { camera_position = square.position:get(), }:play(1)
	s:world_2d():to { camera_position = circle.position:get(), }:play(1)
	s:world_2d():to { camera_position = k.vec2(0, 0), camera_zoom = 1, }:play(1)
end
