return function()
	k.view("3d")
	k.world_3d().camera_position:set(k.vec3(0, 0, 5))

	local picture = k.world_3d():add(k.image_3d{
		source = k.load_image("assets/cat-profile.jpg"),
		size = k.vec2(1.8, 1.8 * 260 / 236),
	})

	picture.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(5, "linear")
end
