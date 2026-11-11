return function()
	local square = k.world_2d():add(k.panel{
		size = k.vec2(256, 128),
		fill = k.color("#008CFF"),
	})

	local left_ball = k.circle{ radius = 20 }
	local right_ball = k.circle{ radius = 28 }
	local top_ball = k.circle{ radius = 36 }
	local bottom_ball = k.circle{ radius = 40 }

	k.world_2d():add(left_ball)
	k.world_2d():add(right_ball)
	k.world_2d():add(top_ball)
	k.world_2d():add(bottom_ball)

	left_ball.position:set(left_ball:right_to(square:left()))
	right_ball.position:set(right_ball:left_to(square:right()))
	top_ball.position:set(top_ball:bottom_to(square:top()))
	bottom_ball.position:set(bottom_ball:top_to(square:bottom()))
end
