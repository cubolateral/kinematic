return function(s)
	local code = s:code {
		code = [=[
local text = "Hello, world!"
print(text)]=],
		language = "lua",
	}
	s:world_2d():add(code)

	code:replace(code:find_all("text"), "greeting"):play(1)
	s:wait(1)
end
