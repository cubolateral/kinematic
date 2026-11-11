return function()
	k.all(function()
		for i, easing in ipairs({ "linear", "cubic_in", "cubic_out", "cubic_in_out" }) do
			local ball = k.circle{
				radius = 32,
				position = k.vec2(-256, (i - 2.5) * 100),
			}
			k.world_2d():add(ball)

			ball:to { position = { x = 256 } }:play(2, easing)
		end
	end)

	k.wait(1)
end
