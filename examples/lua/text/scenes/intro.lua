local font = k.font("assets/HackNerdFontMono-Regular.ttf")

return function(s)
    local text = s:text_2d {
        id = "text",
        font = font,
        text = "Hello, Kinematic.",
        size = 56,
        fill = k.color("#FFFFFF"),
    }
    s:world_2d():add(text)

    text:from {opacity = {0, 1}}:play(0.5)
    s:wait(0.7)

    text:morph("Animate text."):play(1)
    s:wait(0.7)

    text:fade("One scene. Many frames."):play(0.8)
    s:wait(0.7)
    text:to {opacity = 0}:play(0.5)
end
