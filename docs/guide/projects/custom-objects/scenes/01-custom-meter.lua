local progress = k.component("Progress", {
	value = { default = 0, min = 0, max = 1 },
})

local meter = k.object_2d("Meter", {
	components = { progress },

	on_bounds = function(self)
		return k.rect(-256, -8, 512, 16)
	end,

	on_draw = function(self, ctx)
		ctx:rect(-256, -8, 512, 16, k.color("#0A0A0A"))
		ctx:rect(-256, -8, 512 * self.value, 16, self.fill)
	end,
})

return function()
	local first = k.world_2d():add(meter({
		position = k.vec2(0, -64),
		fill = k.color("#FFFFFF"),
	}))

	local second = k.world_2d():add(meter({
		position = k.vec2(0, 64),
		fill = k.color("#008CFF"),
	}))

	k.all(function()
		first.value:to(1):play(2, "linear")
		second.value:to(0.5):play(1, "linear")
	end)

	k.wait(1)
end
