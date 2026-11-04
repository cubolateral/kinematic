return function(s)
	local top = s:circle {
		radius = 32,
		position = k.vec2(-256, -64),
	}
	s:world_2d():add(top)

	local bottom = s:circle {
		radius = 32,
		fill = k.color("#008CFF"),
		position = k.vec2(-256, 64),
	}
	s:world_2d():add(bottom)

	s:all(function()
		s:chain(function()
			top.position.x:to(0):play(1, "linear")
			top.position.x:to(256):play(1, "linear")
		end)
		bottom.position.x:to(256):play(1, "linear")
	end)

	s:wait(1)
end
