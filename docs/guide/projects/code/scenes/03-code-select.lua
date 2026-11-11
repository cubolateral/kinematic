return function()
	local code = k.code{
		code = [=[
local text = "Hello, Kinematic!"
print(text)]=],
		language = "lua",
	}
	k.world_2d():add(code)

	code:select(code:find_all("text")):play(1)
	k.wait(1)
end
