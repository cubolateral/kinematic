return function(s)
    local formula = s:latex_2d {
        id = "formula",
        text = "a^2 + b^2 = c^2",
        size = 64,
        fill = k.color("#FFFFFF"),
    }
    s:world_2d():add(formula)

    formula:from {opacity = {0, 1}}:play(0.5)
    s:wait(0.7)

    formula:morph("3^2 + 4^2 = 5^2"):play(1)
    s:wait(0.7)

    formula:fade("9 + 16 = 25"):play(0.8)
    s:wait(0.7)

    formula:to {fill = k.color("#008CFF")}:play(0.5)
    formula:to {opacity = 0}:play(0.5)
end
