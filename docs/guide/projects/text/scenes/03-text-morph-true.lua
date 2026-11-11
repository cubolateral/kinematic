return function()
	local title = k.text_2d{
		text = "Hello, world!",
		size = 56,
	}
	k.world_2d():add(title)

	title:morph("Hello, wonderful world!", true):play(1)
	k.wait(1)
end
