return function(s)
	for row = 1, 2 do
		local random = k.random(42)

		for i = 1, 128 do
			local x = row == 1 and random:range(-384, 384) or random:normal(0, 100)

			s:world_2d():add(s:circle {
				radius = 4,
				fill = k.color("#008CFF"),
				position = k.vec2(x, (row - 1.5) * 160 + random:range(-25, 25)),
			})
		end
	end
end
