return function(s)
	local ball = s:circle {
		radius = 64,
		position = k.vec2(-256, 0),
	}
	s:world_2d():add(ball)

	ball:to {
		radius = 128,
		position = { x = 256 },
		fill = k.color("#008CFF"),
	}:play(1)

	s:wait(1)
end
