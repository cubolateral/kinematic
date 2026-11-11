return function()
	k.view("3d")
	k.world_3d().camera_position:set(k.vec3(0, 0, 5))

	k.world_3d():add(k.cube{
		position = k.vec3(-1, 0, 0),
	})
	k.world_3d():add(k.cube{
		position = k.vec3(1, 0, -5),
		albedo = k.color("#008CFF"),
	})

	k.wait(1)
	k.world_3d():to { camera_mode = "orthogonal", camera_fov = 5 }:play()
	k.wait(1)
end
