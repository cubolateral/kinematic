return function()
	local code = k.world_2d():add(k.code{
		code = 'print("Hello, world!")',
		language = "lua",
	})

	code:morph([=[local function add(a, b)
    return a + b
end

print(add(1, 1)) -- result: 2]=], true):play(1)
	k.wait(1)
end
