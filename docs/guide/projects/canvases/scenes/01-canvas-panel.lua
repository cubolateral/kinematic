return function()
	k.view("3d")

	local source = k.canvas_2d{
		resolution = { 480, 270 },
		clear = k.color("#0A0A0A"),
	}
	k.root():add(source)

	source:add(k.circle{ radius = 64, fill = k.color("#008CFF") })
	source:add(k.text_2d{ text = "2D" })

	local panel = k.projection_3d{ source = source:texture() }
	k.world_3d():add(panel)

	panel.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(2, "linear")
end
