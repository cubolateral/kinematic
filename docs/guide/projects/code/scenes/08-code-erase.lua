return function()
	local code = k.code{
		code = [=[
local message = "Hello, Kinematic!"
print(message)]=],
		language = "lua",
	}
	k.world_2d():add(code)

	code:erase(code:lines(2)):play(1)
	k.wait(1)
end
