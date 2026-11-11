return function()
	local world = k.world_2d()
	world:add(k.panel{ size = k.vec2(1280, 720), fill = k.color("#000000") })

	local content = world:add(k.group_2d{ id = "content" })

	local walk = k.object_2d("Walk", {
		state = {
			x = 0,
			y = 0,
			random = k.random(42),
			points = {},
		},
		inputs = {},
		on_update = function(self, frame)
			if frame.index % 3 == 0 then
				self.state.x = self.state.x + 15
				self.state.y = self.state.y + self.state.random:range(-18, 18)
				self.state.points[#self.state.points + 1] = { self.state.x, self.state.y }
			end
		end,
		on_bounds = function(self)
			return k.rect(-320, -160, 640, 320)
		end,
		on_draw = function(self, ctx)
			for _, point in ipairs(self.state.points) do
				ctx:circle(point[1] - 300, point[2], 4, self.fill)
			end
			ctx:circle(self.state.x - 300, self.state.y, 10, k.color("#FBBF24"))
		end,
	})
	content:add(k.panel{ size = k.vec2(640, 2), fill = k.color("#383838") })

	local walker = content:add(walk({
		id = "walker",
		fill = k.color("#34D399"),
	}))
	k.wait(2)
	walker:checkpoint()
	k.wait(2)
	walker:to { auto_update = false }:play()

	k.wait(0.6)
	content:to { opacity = 0 }:play(0.5)
	k.wait(0.2)
end
