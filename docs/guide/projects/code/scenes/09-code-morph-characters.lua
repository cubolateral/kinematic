return function()
	local code = k.code{
		code = 'print("Hello, world!")',
		language = "lua",
	}
	k.world_2d():add(code)

	code:morph([=[local function add(a, b)
    return a + b
end

print(add(1, 1)) -- result: 2]=]):play(1)
	k.wait(1)
end
