return function()
	k.view("3d")
	k.world_3d().camera_position:set(k.vec3(0, 0, 5))

	k.world_3d():add(k.directional_light{
		rotation = k.axis_angle(k.vec3(0, 1, 0), -1),
	})

	local group = k.world_3d():add(k.group_3d{})

	group:add(k.pyramid{
		position = k.vec3(-1.5, 0, 0),
		albedo = k.color("#FF4040"),
	})
	group:add(k.cube{
		albedo = k.color("#40FF80"),
	})
	group:add(k.sphere{
		position = k.vec3(1.5, 0, 0),
		albedo = k.color("#4080FF"),
	})
	group.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
end
