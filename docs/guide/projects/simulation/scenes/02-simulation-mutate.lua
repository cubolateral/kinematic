local particle = k.object_2d("Particle", {
	state = { x = -240, trail = {} },
	components = { k.component("Custom", {
		speed = { default = 160, min = 0 },
	}) },
	inputs = { speed = "speed" },

	on_update = function(self, frame,inputs)
		self.state.x = self.state.x + inputs.speed * frame.dt
		self.state.trail[#self.state.trail + 1] = self.state.x
		if #self.state.trail > 120 then
			table.remove(self.state.trail, 1)
		end
	end,

	on_bounds = function(self)
		return k.rect(-270, -30, 810, 60)
	end,

	on_draw = function(self, ctx)
		for _, x in ipairs(self.state.trail) do
			ctx:circle(x, 0, 2, k.color("#008CFF"))
		end
		ctx:circle(self.state.x, 0, 18, self.fill)
	end,
})

return function()
	local ball = k.world_2d():add(particle({ fill = k.color("#FFFFFF") }))

	k.wait(1)
	ball:mutate(function(state)
		state.x = -240
		state.trail = {}
	end)
	k.wait(1)
end
