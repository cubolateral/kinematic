return function()
	k.view("3d")

	local label = k.world_3d():add(k.text_3d{
		text = "Hello, 3D world!",
		size = 0.3,
	})

	label.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(5, "linear")
end
