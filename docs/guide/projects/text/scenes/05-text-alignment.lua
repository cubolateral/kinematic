return function()
	k.world_2d():add(k.text_2d{
		text = "Left align\ntext",
		size = 32,
		align = -1,
		position = { x = -300 },
	})
	k.world_2d():add(k.text_2d{
		text = "Middle align\ntext",
		size = 32,
		align = 0,
	})
	k.world_2d():add(k.text_2d{
		text = "Right align\ntext",
		size = 32,
		align = 1,
		position = { x = 300 },
	})
end
