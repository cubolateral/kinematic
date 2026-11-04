return function(s)
	s:view("3d")

	local label = s:text_3d {
		text = "Hello, 3D world!",
		size = 0.3,
	}
	s:world_3d():add(label)

	label.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(5, "linear")
end
