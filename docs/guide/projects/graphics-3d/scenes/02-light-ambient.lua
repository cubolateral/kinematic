return function()
	k.view("3d")
	k.world_3d().camera_position:set(k.vec3(0, 0, 5))

	local group = k.world_3d():add(k.group_3d{})

	group:add(k.pyramid{ position = k.vec3(-1.5, 0, 0), })
	group:add(k.cube{})
	group:add(k.sphere{ position = k.vec3(1.5, 0, 0), })

	k.loop(function()
		group.rotation:rotate(k.vec3(0, 1, 0), math.pi * 2):play(6.5, "linear")
	end)

	k.world_3d().ambient_intensity:to(0):play(1)
	k.wait(0.5)
	k.world_3d().ambient_intensity:to(1):play(1)

	k.world_3d().ambient_color:to(k.color("#FF0000")):play(1)
	k.world_3d().ambient_color:to(k.color("#00FF00")):play(1)
	k.world_3d().ambient_color:to(k.color("#0000FF")):play(1)

	k.world_3d():to {
		ambient_color = k.color("#FFFFFF"),
		ambient_intensity = 0.4,
	}:play(1)
end
