return function()
	for i, color in ipairs({ "#FF4040", "#40FF80", "#4080FF" }) do
		k.world_2d():add(k.circle{
			radius = 64,
			position = k.vec2((i - 2) * 192, 0),
			fill = k.color(color),
		})
	end
end
