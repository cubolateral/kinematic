return function(s)
	for column = 1, 2 do
		local group = s:group_2d {
			position = k.vec2((column - 1.5) * 384, 0)
		}
		s:world_2d():add(group)

		local ball = s:circle {
			radius = 64,
			position = k.vec2(-32, 0),
		}
		group:add(ball)

		group:add(s:circle {
			radius = 64,
			position = k.vec2(32, 0),
			fill = k.color("#008CFF"),
		})

		if column == 2 then
			ball:move_to_top()
		end
	end
end
