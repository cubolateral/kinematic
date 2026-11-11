return function()
	local formula = k.world_2d():add(k.latex_2d{
		text = [[\text{mean velocity} = \frac{\Delta\text{position}}{\Delta\text{time}}]],
		size = 56,
	})

	formula:morph(
		[[\text{mean acceleration} = \frac{\Delta\text{velocity}}{\Delta\text{time}}]],
		false):play(1)

	k.wait(1)
end
