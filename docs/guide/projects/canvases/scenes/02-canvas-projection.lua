return function(s)
	local source = s:canvas_3d {
		resolution = { 480, 270 },
		clear = k.color("#0A0A0A"),
	}
	s:root():add(source)

	local group = s:group_3d {}
	source:add(group)

	group:add(s:sphere { albedo = k.color("#008CFF"), unlit = true })
	group:add(s:text_3d { text = "3D", size = 0.5, position = { z = 0.5 } })

	s:world_2d():add(s:projection_2d {
		source = source:texture(),
		size = k.vec2(480, 270),
	})
	group.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(2, "linear")
end
