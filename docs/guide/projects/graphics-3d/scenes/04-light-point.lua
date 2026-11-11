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

	local light = k.point_light{
		position = k.vec3(-1.75, 0.75, 1.75),
		intensity = 5,
		quadratic = 0.25,
	}
	k.world_3d():add(light)

	local marker = k.sphere{
		radius = 0.05,
		unlit = true,
	}
	k.world_3d():add(marker)

	marker:signal(function(object)
		object.position:set(light.position:get())
	end)

	light.position.x:to(1.75):play(1)
	light.position.x:to(-1.75):play(1)
end
