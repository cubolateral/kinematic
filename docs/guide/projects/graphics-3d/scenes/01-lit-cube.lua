return function()
	k.view("3d")

	local cube = k.world_3d():add(k.cube{})

	cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
end
