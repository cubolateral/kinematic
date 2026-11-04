return function(s)
	s:all(function()
		for i, easing in ipairs({ "linear", "cubic_in", "cubic_out", "cubic_in_out" }) do
			local ball = s:circle {
				radius = 32,
				position = k.vec2(-256, (i - 2.5) * 100),
			}
			s:world_2d():add(ball)

			ball:to { position = { x = 256 } }:play(2, easing)
		end
	end)

	s:wait(1)
end
