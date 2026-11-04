return function(s)
	local ball = s:circle {
		radius = 64,
		position = k.vec2(-256, 64),
	}
	s:world_2d():add(ball)

	s:loop(function()
		ball.position.y:to(-64):play(0.5)
		ball.position.y:to(64):play(0.5)
	end)

	s:parallel(function()
		ball.position.x:to(256):play(2, "linear")
	end)

	s:wait(2)
end
