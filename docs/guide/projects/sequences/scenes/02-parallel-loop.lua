return function()
	local ball = k.world_2d():add(k.circle{
		radius = 64,
		position = k.vec2(-256, 64),
	})

	k.loop(function()
		ball.position.y:to(-64):play(0.5)
		ball.position.y:to(64):play(0.5)
	end)

	k.parallel(function()
		ball.position.x:to(256):play(2, "linear")
	end)

	k.wait(2)
end
