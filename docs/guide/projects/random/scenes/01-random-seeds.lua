return function(s)
	for row, seed in ipairs({ 42, 42, 91 }) do
		local random = k.random(seed)

		for column = 1, 9 do
			s:world_2d():add(s:circle {
				radius = random:range(8, 32),
				position = k.vec2((column - 5) * 85, (row - 2) * 140),
			})
		end
	end
end
