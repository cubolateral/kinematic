return function()
	local ball = k.circle{
		radius = 64,
		position = k.vec2(-256, 0),
	}
	k.world_2d():add(ball)

	ball.position.x:to(256):play(2, "linear")
	k.wait(1)
end
