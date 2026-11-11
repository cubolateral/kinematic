return function()
	local source = k.root():add(k.canvas_3d{
		resolution = { 480, 270 },
		clear = k.color("#0A0A0A"),
	})

	local group = source:add(k.group_3d{})

	group:add(k.sphere{ albedo = k.color("#008CFF"), unlit = true })
	group:add(k.text_3d{ text = "3D", size = 0.5, position = { z = 0.5 } })

	k.world_2d():add(k.projection_2d{
		source = source:texture(),
		size = k.vec2(480, 270),
	})
	group.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(2, "linear")
end
