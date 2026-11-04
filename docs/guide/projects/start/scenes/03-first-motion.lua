return function(s)
	local ball = s:circle {
		radius = 64,
		position = k.vec2(-256, 0),
	}
	s:world_2d():add(ball)

	ball:to { position = k.vec2(256, 0) }:play(1)
	s:wait(1)
end
