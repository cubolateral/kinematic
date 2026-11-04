return function(s)
	s:view("3d")
	s:world_3d().camera_position:set(k.vec3(0, 0, 5))

	s:world_3d():add(s:cube {
		position = k.vec3(-1, 0, 0),
	})
	s:world_3d():add(s:cube {
		position = k.vec3(1, 0, -5),
		albedo = k.color("#008CFF"),
	})

	s:wait(1)
	s:world_3d():to { camera_mode = "orthogonal", camera_fov = 5 }:play()
	s:wait(1)
end
