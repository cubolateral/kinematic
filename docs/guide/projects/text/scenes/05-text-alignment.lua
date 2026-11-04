return function(s)
	s:world_2d():add(s:text_2d {
		text = "Left align\ntext",
		size = 32,
		align = -1,
		position = { x = -300 },
	})
	s:world_2d():add(s:text_2d {
		text = "Middle align\ntext",
		size = 32,
		align = 0,
	})
	s:world_2d():add(s:text_2d {
		text = "Right align\ntext",
		size = 32,
		align = 1,
		position = { x = 300 },
	})
end
