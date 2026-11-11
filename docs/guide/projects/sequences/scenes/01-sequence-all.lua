return function()
	local top = k.world_2d():add(k.circle{
		radius = 32,
		position = k.vec2(-256, -64),
	})

	local bottom = k.world_2d():add(k.circle{
		radius = 32,
		fill = k.color("#008CFF"),
		position = k.vec2(-256, 64),
	})

	k.all(function()
		k.chain(function()
			top.position.x:to(0):play(1, "linear")
			top.position.x:to(256):play(1, "linear")
		end)
		bottom.position.x:to(256):play(1, "linear")
	end)

	k.wait(1)
end
