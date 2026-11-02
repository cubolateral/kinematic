return function(s)
	local world = s:world_2d()
	world:add(s:rect { size = k.vec2(1280, 720), fill = k.color("#000000") })

	local content = s:group_2d { id = "content" }
	world:add(content)

	local walk = k.object {
		state = {
			x = 0,
			y = 0,
			random = k.random(42),
			points = {},
		},
		inputs = {},
		update = function(state, frame)
			if frame.index % 3 == 0 then
				state.x = state.x + 15
				state.y = state.y + state.random:range(-18, 18)
				state.points[#state.points + 1] = { state.x, state.y }
			end
		end,
		bounds = function()
			return { x = -320, y = -160, width = 640, height = 320 }
		end,
		draw = function(ctx, properties, state)
			for _, point in ipairs(state.points) do
				ctx:circle(point[1] - 300, point[2], 4, properties.fill)
			end
			ctx:circle(state.x - 300, state.y, 10, k.color("#FBBF24"))
		end,
	}
	content:add(s:rect { size = k.vec2(640, 2), fill = k.color("#383838") })

	local walker = s:object(walk, {
		id = "walker",
		fill = k.color("#34D399"),
	})
	content:add(walker)
	s:wait(2)
	walker:checkpoint()
	s:wait(2)
	walker:to { auto_update = false }:play()

	s:wait(0.6)
	content:to { opacity = 0 }:play(0.5)
	s:wait(0.2)
end
