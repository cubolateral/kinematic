return function(s)
	local square = s:rect {
		size = k.vec2(256, 128),
		fill = k.color("#008CFF"),
	}
	s:world_2d():add(square)

	local left_ball = s:circle { radius = 20 }
	local right_ball = s:circle { radius = 28 }
	local top_ball = s:circle { radius = 36 }
	local bottom_ball = s:circle { radius = 40 }

	s:world_2d():add(left_ball)
	s:world_2d():add(right_ball)
	s:world_2d():add(top_ball)
	s:world_2d():add(bottom_ball)

	left_ball.position:set(left_ball:right_to(square:left()))
	right_ball.position:set(right_ball:left_to(square:right()))
	top_ball.position:set(top_ball:bottom_to(square:top()))
	bottom_ball.position:set(bottom_ball:top_to(square:bottom()))
end
