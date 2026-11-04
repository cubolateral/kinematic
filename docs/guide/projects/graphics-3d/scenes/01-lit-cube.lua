return function(s)
	s:view("3d")

	local cube = s:cube {}
	s:world_3d():add(cube)

	cube.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
end
