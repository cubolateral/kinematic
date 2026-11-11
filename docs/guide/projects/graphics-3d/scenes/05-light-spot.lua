return function()
	k.view("3d")
	k.world_3d().camera_position:set(k.vec3(0, 0, 5))

	k.world_3d():add(k.pyramid{
		position = k.vec3(-1.5, 0, 0),
		albedo = k.color("#FF4040"),
	})
	k.world_3d():add(k.cube{
		albedo = k.color("#40FF80"),
	})
	k.world_3d():add(k.sphere{
		position = k.vec3(1.5, 0, 0),
		albedo = k.color("#4080FF"),
	})

	local light = k.world_3d():add(k.spot_light{
		position = k.vec3(0, 0, 2),
		rotation = k.axis_angle(k.vec3(0, 1, 0), -1),
		intensity = 5,
		quadratic = 0.1,
		cutoff = 0.4,
	})

	light.rotation:to(k.axis_angle(k.vec3(0, 1, 0), 1)):play(2)
	light.rotation:to(k.axis_angle(k.vec3(0, 1, 0), -1)):play(2)
end
