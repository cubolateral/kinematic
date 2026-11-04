return function(s)
	local code = s:code {
		code = [=[
local message = "Hello, Kinematic!"
print(message)]=],
		language = "lua",
	}
	s:world_2d():add(code)

	code:erase(code:lines(2)):play(1)
	s:wait(1)
end
