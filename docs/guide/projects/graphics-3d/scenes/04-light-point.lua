return function(s)
	s:view("3d")
	s:world_3d().camera_position:set(k.vec3(0, 0, 5))

	s:world_3d():add(s:pyramid {
		position = k.vec3(-1.5, 0, 0),
		albedo = k.color("#FF4040"),
	})
	s:world_3d():add(s:cube {
		albedo = k.color("#40FF80"),
	})
	s:world_3d():add(s:sphere {
		position = k.vec3(1.5, 0, 0),
		albedo = k.color("#4080FF"),
	})

	local light = s:point_light {
		position = k.vec3(-1.75, 0.75, 1.75),
		intensity = 5,
		quadratic = 0.25,
	}
	s:world_3d():add(light)

	local marker = s:sphere {
		radius = 0.05,
		unlit = true,
	}
	s:world_3d():add(marker)

	marker:signal(function(object)
		object.position:set(light.position:get())
	end)

	light.position.x:to(1.75):play(1)
	light.position.x:to(-1.75):play(1)
end
