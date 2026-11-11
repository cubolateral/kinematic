local font = k.load_font("assets/HackNerdFontMono-Regular.ttf")

return function()
    local text = k.text_2d{
        id = "text",
        font = font,
        text = "Hello, Kinematic.",
        size = 56,
        fill = k.color("#FFFFFF"),
    }
    k.world_2d():add(text)

    text:from {opacity = {0, 1}}:play(0.5)
    k.wait(0.7)

    text:morph("Animate text."):play(1)
    k.wait(0.7)

    text:fade("One scene. Many frames."):play(0.8)
    k.wait(0.7)
    text:to {opacity = 0}:play(0.5)
end
