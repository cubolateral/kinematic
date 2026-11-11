return function()
	local code = k.world_2d():add(k.code{
		code = 'print("Hello, world!")',
		language = "lua",
	})

	code:replace(code:find("world"), "Kinematic"):play(1)
	k.wait(1)
end
