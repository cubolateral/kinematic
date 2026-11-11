return function()
	local code = k.code{
		code = 'print("Hello, world!")',
		language = "lua",
	}
	k.world_2d():add(code)

	code:replace(code:find("world"), "Kinematic"):play(1)
	k.wait(1)
end
