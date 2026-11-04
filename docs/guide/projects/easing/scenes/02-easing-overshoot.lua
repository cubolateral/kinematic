return function(s)
	s:all(function()
		for i, easing in ipairs({ "back_out", "elastic_out", "bounce_out" }) do
			local ball = s:circle {
				radius = 32,
				position = k.vec2(-256, (i - 2) * 128),
			}
			s:world_2d():add(ball)

			ball:to { position = { x = 256 } }:play(2, easing)
		end
	end)

	s:wait(1)
end
