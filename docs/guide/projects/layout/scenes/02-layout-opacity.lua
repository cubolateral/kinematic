return function(s)
	local group_opacity = s:group_2d {
		position = k.vec2(-192, 0),
		opacity = 0.5,
	}
	s:world_2d():add(group_opacity)

	local child_opacity = s:group_2d {
		position = k.vec2(192, 0),
	}
	s:world_2d():add(child_opacity)

	for _, circle in ipairs {
		{ position = k.vec2(0, -32),  fill = k.color("#FF4040") },
		{ position = k.vec2(-32, 32), fill = k.color("#40FF80") },
		{ position = k.vec2(32, 32),  fill = k.color("#4080FF") },
	} do
		group_opacity:add(s:circle {
			radius = 64,
			position = circle.position,
			fill = circle.fill,
		})

		child_opacity:add(s:circle {
			radius = 64,
			position = circle.position,
			fill = circle.fill,
			opacity = 0.5,
		})
	end
end
