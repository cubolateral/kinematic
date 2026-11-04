return function(s)
	s:view("3d")

	local label = s:latex_3d {
		text = [[e^{i\pi} + 1 = 0]],
		size = 0.5,
	}
	s:world_3d():add(label)

	label.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(5, "linear")
end
