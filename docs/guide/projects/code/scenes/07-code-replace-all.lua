return function()
	local code = k.code{
		code = [=[
local text = "Hello, world!"
print(text)]=],
		language = "lua",
	}
	k.world_2d():add(code)

	code:replace(code:find_all("text"), "greeting"):play(1)
	k.wait(1)
end
