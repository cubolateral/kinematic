return function(s)
	s:world_2d():add(s:code {
		code = 'print("Hello, world!")',
		language = "lua",
	})
end
