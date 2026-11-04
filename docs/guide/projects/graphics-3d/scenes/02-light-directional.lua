return function(s)
	s:view("3d")
	local world = s:world_3d()
	world.camera_position:set(k.vec3(0, 0, 5))

	local light = s:directional_light {
		rotation = k.axis_angle(k.vec3(0, 1, 0), -1),
	}
	world:add(light)

	local group = s:group_3d {}
	world:add(group)

	group:add(s:pyramid {
		position = k.vec3(-1.5, 0, 0),
		albedo = k.color("#FF4040"),
	})
	group:add(s:cube {
		albedo = k.color("#40FF80"),
	})
	group:add(s:sphere {
		position = k.vec3(1.5, 0, 0),
		albedo = k.color("#4080FF"),
	})
	group.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(3, "linear")
end
