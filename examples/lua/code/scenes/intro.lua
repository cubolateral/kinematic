local font = k.load_font("assets/HackNerdFontMono-Regular.ttf")

return function()
    local code = k.world_2d():add(k.code{
        id = "code",
        font = font,
        language = "lua",
        size = 40,
        code = "local value = 1\nprint(value)",
    })

    code:from {opacity = {0, 1}}:play(0.5)
    code:highlight(code:find_all("value")):play(0.5)
    k.wait(0.5)

    code:highlight(nil):play(0.4)
    code:replace(code:find("1"), "42"):play(0.8)
    code:append("local doubled = value * 2"):play(0.8)

    code:select(code:lines(3)):play(0.5)
    k.wait(0.5)

    code:select(nil):play(0.4)
    local function_code = [=[
local function double(value)
    return value * 2
end

print(double(42))]=]
    code:morph(function_code, true):play(1)
    k.wait(0.7)
    code:to {opacity = 0}:play(0.5)
end
