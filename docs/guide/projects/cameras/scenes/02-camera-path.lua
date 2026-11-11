local function point(angle)
	return k.vec2(240 * math.cos(angle), 100 * math.sin(angle))
end

return function()
	k.world_2d().camera_zoom:set(1.5)

	for i = 0, 47 do
		k.world_2d():add(k.circle{
			radius = 2,
			position = point(i * math.pi * 2 / 48),
			fill = k.color("#4080FF"),
		})
	end

	local phase = k.track(0)

	k.world_2d():signal(function(canvas)
		canvas.camera_position:set(point(phase:get()))
	end)

	phase:to(math.pi * 2):play(2, "linear")
end
