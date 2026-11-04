return function(s)
	local target = s:circle {
		radius = 32,
		position = k.vec2(-256, -64),
	}
	s:world_2d():add(target)

	local increment = s:circle {
		radius = 32,
		fill = k.color("#008CFF"),
		position = k.vec2(-256, 64),
	}
	s:world_2d():add(increment)

	s:all(function()
		s:chain(function()
			target.position.x:to(0):play(1, "linear")
			target.position.x:to(0):play(1, "linear")
		end)
		s:chain(function()
			increment.position.x:by(256):play(1, "linear")
			increment.position.x:by(256):play(1, "linear")
		end)
	end)

	s:wait(1)
end
