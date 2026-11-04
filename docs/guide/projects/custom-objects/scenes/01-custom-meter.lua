local progress = k.component {
	value = { default = 0, min = 0, max = 1 },
}

local meter = k.object {
	name = "Meter",
	components = { progress },

	bounds = function()
		return { x = -256, y = -8, width = 512, height = 16 }
	end,

	draw = function(ctx, properties)
		ctx:rect(-256, -8, 512, 16, k.color("#0A0A0A"))
		ctx:rect(-256, -8, 512 * properties.value, 16, properties.fill)
	end,
}

return function(s)
	local first = s:object(meter, {
		position = k.vec2(0, -64),
		fill = k.color("#FFFFFF"),
	})
	s:world_2d():add(first)

	local second = s:object(meter, {
		position = k.vec2(0, 64),
		fill = k.color("#008CFF"),
	})
	s:world_2d():add(second)

	s:all(function()
		first.value:to(1):play(2, "linear")
		second.value:to(0.5):play(1, "linear")
	end)

	s:wait(1)
end
