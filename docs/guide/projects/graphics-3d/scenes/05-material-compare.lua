return function(s)
	s:view("3d")
	s:world_3d().camera_position:set(k.vec3(0, 0, 5))

	s:world_3d():add(s:sphere {
		position = k.vec3(-1.5, 0, 0),
	})
	s:world_3d():add(s:sphere {
		metallic = 1,
		roughness = 0.75,
	})
	s:world_3d():add(s:sphere {
		position = k.vec3(1.5, 0, 0),
		unlit = true,
	})
end
