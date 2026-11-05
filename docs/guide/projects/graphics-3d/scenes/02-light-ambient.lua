return function(s)
	s:view("3d")
	s:world_3d().camera_position:set(k.vec3(0, 0, 5))

	local group = s:group_3d {}
	s:world_3d():add(group)

	group:add(s:pyramid { position = k.vec3(-1.5, 0, 0), })
	group:add(s:cube {})
	group:add(s:sphere { position = k.vec3(1.5, 0, 0), })

	s:loop(function()
		group.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(6.5, "linear")
	end)

	s:world_3d().ambient_intensity:to(0):play(1)
	s:wait(0.5)
	s:world_3d().ambient_intensity:to(1):play(1)

	s:world_3d().ambient_color:to(k.color("#FF0000")):play(1)
	s:world_3d().ambient_color:to(k.color("#00FF00")):play(1)
	s:world_3d().ambient_color:to(k.color("#0000FF")):play(1)

	s:world_3d():to {
		ambient_color = k.color("#FFFFFF"),
		ambient_intensity = 0.4,
	}:play(1)
end
