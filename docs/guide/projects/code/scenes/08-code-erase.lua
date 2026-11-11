return function()
	local code = k.world_2d():add(k.code{
		code = [=[
local message = "Hello, Kinematic!"
print(message)]=],
		language = "lua",
	})

	code:erase(code:lines(2)):play(1)
	k.wait(1)
end
