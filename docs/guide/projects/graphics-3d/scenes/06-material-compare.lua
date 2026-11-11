return function()
	k.view("3d")
	k.world_3d().camera_position:set(k.vec3(0, 0, 5))

	k.world_3d():add(k.sphere{
		position = k.vec3(-1.5, 0, 0),
	})
	k.world_3d():add(k.sphere{
		metallic = 1,
		roughness = 0.75,
	})
	k.world_3d():add(k.sphere{
		position = k.vec3(1.5, 0, 0),
		unlit = true,
	})
end
