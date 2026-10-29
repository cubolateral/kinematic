return function(s)
	local code = s:code {
		id = "code_example",
		language = "lua",
		size = 42,
		line_spacing = 1.2,
		code = 'local value = 1\nprint(value)',
		highlight_color = k.color("#FFCC0066"),
	}

	s:world_2d():add(code)
	s:wait(0.5)

	code:select(code:find_all("value")):play(1)
	code:highlight(code:find_all("value")):play(1)
	s:wait(1)

	s:all(function()
		code:select(nil):play(1)
		code:highlight({}):play(1)
	end)

	code:replace(code:find("1"), "42"):play(1)
	code:append("-- Result."):play(1)
	code:prepend("-- Example."):play(1)
	code:insert({ 3, 1 }, "local doubled = value * 2\n"):play(1)
	code:replace(code:find("value"), "doubled"):play(1)
	s:wait(0.5)

	-- Combines literal searches with a complete line.
	code:select({ code:find("42"), code:find_all("value"), code:lines(3) }):play(1)
	s:wait(1)
	code:select(nil):play(1)

	code:morph('local function double(value)\n    return value * 2\nend\n\nprint(double(42))'):play(1)
	s:wait(1)
	code:morph('print("Done!")', true):play(1)
	code:erase(code:find('"Done!"')):play(1)
	s:wait(1)
end
