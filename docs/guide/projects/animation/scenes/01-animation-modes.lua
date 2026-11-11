return function()
	local target = k.world_2d():add(k.circle{
		radius = 32,
		position = k.vec2(-256, -64),
	})

	local increment = k.world_2d():add(k.circle{
		radius = 32,
		fill = k.color("#008CFF"),
		position = k.vec2(-256, 64),
	})

	k.all(function()
		k.chain(function()
			target.position.x:to(0):play(1, "linear")
			target.position.x:to(0):play(1, "linear")
		end)
		k.chain(function()
			increment.position.x:by(256):play(1, "linear")
			increment.position.x:by(256):play(1, "linear")
		end)
	end)

	k.wait(1)
end
