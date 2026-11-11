return function()
	for i, direction in ipairs({ "horizontal", "vertical", "horizontal_reverse" }) do
		local row = k.world_2d():add(k.container_2d{
			direction = direction,
			gap = k.vec2(16, 16),
			position = k.vec2((i - 2) * 256, 0),
		})

		for _, color in ipairs({ "#FF4040", "#40FF80", "#4080FF" }) do
			row:add(k.panel{ size = k.vec2(64, 64), fill = k.color(color) })
		end
	end
end
