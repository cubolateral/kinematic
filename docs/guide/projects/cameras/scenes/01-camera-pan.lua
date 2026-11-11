return function()
	local triangle = k.triangle{
		size = k.vec2(128, 128),
		position = k.vec2(-192, 0),
		fill = k.color("#FF4040"),
	}
	k.world_2d():add(triangle)

	local square = k.square{
		size = k.vec2(128, 128),
		fill = k.color("#40FF80"),
	}
	k.world_2d():add(square)

	local circle = k.circle{
		radius = 64,
		position = k.vec2(192, 0),
		fill = k.color("#4080FF"),
	}
	k.world_2d():add(circle)

	k.world_2d():to { camera_position = triangle.position:get(), camera_zoom = 2.5, }:play(1)
	k.world_2d():to { camera_position = square.position:get(), }:play(1)
	k.world_2d():to { camera_position = circle.position:get(), }:play(1)
	k.world_2d():to { camera_position = k.vec2(0, 0), camera_zoom = 1, }:play(1)
end
