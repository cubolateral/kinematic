return function()
	local title = k.world_2d():add(k.text_2d{
		text = "Hello, world!",
		size = 56,
	})

	title:fade("This is a faded text!"):play(1)
	k.wait(1)
end
