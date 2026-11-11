return function()
	local code = k.world_2d():add(k.code{
		code = [=[
local text = "Hello, Kinematic!"
print(text)]=],
		language = "lua",
	})

	code:select(code:find_all("text")):play(1)
	k.wait(1)
end
