return function()
	k.view("3d")

	local label = k.latex_3d{
		text = [[e^{i\pi} + 1 = 0]],
		size = 0.5,
	}
	k.world_3d():add(label)

	label.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(5, "linear")
end
