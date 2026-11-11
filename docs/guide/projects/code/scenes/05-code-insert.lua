return function()
	local code = k.world_2d():add(k.code{
		code = 'print("Hello!")',
		language = "lua",
	})

	code:insert({ 1, 13 }, ", Kinematic"):play(1)
	k.wait(1)
end
