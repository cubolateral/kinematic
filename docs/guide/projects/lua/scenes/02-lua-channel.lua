return function(s)
	local ball = s:circle {
		radius = 64,
		position = k.vec2(-256, 0),
	}
	s:world_2d():add(ball)

	ball.position.x:to(256):play(2, "linear")
	s:wait(1)
end
