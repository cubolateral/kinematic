return function(s)
	local code = s:code {
		code = [=[
local text = "Hello, Kinematic!"
print(text)]=],
		language = "lua",
	}
	s:world_2d():add(code)

	code:highlight(code:find_all("text")):play(1)
	s:wait(1)
end
