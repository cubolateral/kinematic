local particle = k.object {
	state = { x = -240, trail = {} },
	components = { k.component {
		speed = { default = 160, min = 0 },
	} },
	inputs = { speed = "speed" },

	update = function(state, frame, inputs)
		state.x = state.x + inputs.speed * frame.dt
		state.trail[#state.trail + 1] = state.x
		if #state.trail > 120 then
			table.remove(state.trail, 1)
		end
	end,

	bounds = function()
		return { x = -270, y = -30, width = 810, height = 60 }
	end,

	draw = function(ctx, properties, state)
		for _, x in ipairs(state.trail) do
			ctx:circle(x, 0, 2, k.color("#008CFF"))
		end
		ctx:circle(state.x, 0, 18, properties.fill)
	end,
}

return function(s)
	local ball = s:object(particle, { fill = k.color("#FFFFFF") })
	s:world_2d():add(ball)

	s:wait(1)
	ball.speed:to(60):play(1, "linear")
	s:wait(0.5)
end
