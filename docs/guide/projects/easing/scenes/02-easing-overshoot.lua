return function()
	k.all(function()
		for i, easing in ipairs({ "back_out", "elastic_out", "bounce_out" }) do
			local ball = k.world_2d():add(k.circle{
				radius = 32,
				position = k.vec2(-256, (i - 2) * 128),
			})

			ball:to { position = { x = 256 } }:play(2, easing)
		end
	end)

	k.wait(1)
end
