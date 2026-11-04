return function(s)
	local code = s:code {
		code = 'print("Hello!")',
		language = "lua",
	}
	s:world_2d():add(code)

	code:insert({ 1, 13 }, ", Kinematic"):play(1)
	s:wait(1)
end
