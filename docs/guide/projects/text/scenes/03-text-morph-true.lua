return function(s)
	local title = s:text_2d {
		text = "Hello, world!",
		size = 56,
	}
	s:world_2d():add(title)

	title:morph("Hello, wonderful world!", true):play(1)
	s:wait(1)
end
