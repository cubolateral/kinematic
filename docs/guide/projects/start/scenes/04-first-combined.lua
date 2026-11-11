return function()
	local ball = k.world_2d():add(k.circle{
		radius = 64,
		position = k.vec2(-256, 0),
	})

	ball:to {
		radius = 128,
		position = { x = 256 },
		fill = k.color("#008CFF"),
	}:play(1)

	k.wait(1)
end
