return function(s)
	local code = s:code {
		code = 'print("Hello, world!")',
		language = "lua",
	}
	s:world_2d():add(code)

	code:replace(code:find("world"), "Kinematic"):play(1)
	s:wait(1)
end
