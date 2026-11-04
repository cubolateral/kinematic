return function(s)
	local code = s:code {
		code = 'print("Hello, world!")',
		language = "lua",
	}
	s:world_2d():add(code)

	code:morph([=[local function add(a, b)
    return a + b
end

print(add(1, 1)) -- result: 2]=]):play(1)
	s:wait(1)
end
