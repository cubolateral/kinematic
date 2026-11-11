return function()
	local group_opacity = k.world_2d():add(k.group_2d{
		position = k.vec2(-192, 0),
		opacity = 0.5,
	})

	local child_opacity = k.world_2d():add(k.group_2d{
		position = k.vec2(192, 0),
	})

	for _, circle in ipairs {
		{ position = k.vec2(0, -32),  fill = k.color("#FF4040") },
		{ position = k.vec2(-32, 32), fill = k.color("#40FF80") },
		{ position = k.vec2(32, 32),  fill = k.color("#4080FF") },
	} do
		group_opacity:add(k.circle{
			radius = 64,
			position = circle.position,
			fill = circle.fill,
		})

		child_opacity:add(k.circle{
			radius = 64,
			position = circle.position,
			fill = circle.fill,
			opacity = 0.5,
		})
	end
end
