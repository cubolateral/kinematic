return function()
	local ball = k.world_2d():add(k.circle{
		radius = 64,
		position = k.vec2(-256, 0),
	})

	ball:to { position = k.vec2(256, 0) }:play(1)
	k.wait(1)
end
