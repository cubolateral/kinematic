return function()
	local code = k.world_2d():add(k.code{
		code = 'print("Hello, world!")',
		language = "lua",
	})

	code:highlight({ { 1, 1 }, { 1, 6 } }):play(1)
	k.wait(1)
end
