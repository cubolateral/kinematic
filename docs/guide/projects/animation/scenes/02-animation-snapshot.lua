return function(s)
	local ball = s:circle { radius = 32 }
	s:world_2d():add(ball)

	ball.position.x:set(-256)
	local pose = ball:snapshot()

	ball:to { position = { x = 256 }, radius = 64 }:play(1)
	ball:restore(pose):play(1)
	ball:reset():play(1)

	s:wait(1)
end
