return function(s)
	local formula = s:latex_2d {
		text = [[\text{mean velocity} = \frac{\Delta\text{position}}{\Delta\text{time}}]],
		size = 56,
	}
	s:world_2d():add(formula)

	formula:morph(
		[[\text{mean acceleration} = \frac{\Delta\text{velocity}}{\Delta\text{time}}]],
		true):play(1)

	s:wait(1)
end
