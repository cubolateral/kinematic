return function()
	local ball = k.world_2d():add(k.circle{ radius = 32 })

	ball.position.x:set(-256)
	local pose = ball:snapshot()

	ball:to { position = { x = 256 }, radius = 64 }:play(1)
	ball:restore(pose):play(1)
	ball:reset():play(1)

	k.wait(1)
end
