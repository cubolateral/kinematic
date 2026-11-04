return function(s)
	local title = s:text_2d {
		text = "Hello, world!",
		size = 56,
	}
	s:world_2d():add(title)

	title:fade("This is a faded text!"):play(1)
	s:wait(1)
end
