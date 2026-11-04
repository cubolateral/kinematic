return function(s)
	s:view("3d")
	s:world_3d().camera_position:set(k.vec3(0, 0, 5))

	local picture = s:image_3d {
		source = k.image("assets/cat-profile.jpg"),
		size = k.vec2(1.8, 1.8 * 260 / 236),
	}
	s:world_3d():add(picture)

	picture.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(5, "linear")
end
