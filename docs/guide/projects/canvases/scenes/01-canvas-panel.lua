return function(s)
	s:view("3d")

	local source = s:canvas_2d {
		resolution = { 480, 270 },
		clear = k.color("#0A0A0A"),
	}
	s:root():add(source)

	source:add(s:circle { radius = 64, fill = k.color("#008CFF") })
	source:add(s:text_2d { text = "2D" })

	local panel = s:projection_3d { source = source:texture() }
	s:world_3d():add(panel)

	panel.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(2, "linear")
end
