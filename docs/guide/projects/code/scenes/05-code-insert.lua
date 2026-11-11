return function()
	local code = k.code{
		code = 'print("Hello!")',
		language = "lua",
	}
	k.world_2d():add(code)

	code:insert({ 1, 13 }, ", Kinematic"):play(1)
	k.wait(1)
end
