return function(s)
	local code = s:code {
		code = 'print("Hello, world!")',
		language = "lua",
	}
	s:world_2d():add(code)

	code:highlight({ { 1, 1 }, { 1, 6 } }):play(1)
	s:wait(1)
end
