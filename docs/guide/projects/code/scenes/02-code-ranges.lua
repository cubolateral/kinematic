return function()
	local code = k.code{
		code = 'print("Hello, world!")',
		language = "lua",
	}
	k.world_2d():add(code)

	code:highlight({ { 1, 1 }, { 1, 6 } }):play(1)
	k.wait(1)
end
