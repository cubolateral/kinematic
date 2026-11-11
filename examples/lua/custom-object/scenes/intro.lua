local progress = k.component("Progress", {
    value = {default = 0, min = 0, max = 1},
    width = {default = 520, min = 1, max = 800},
    height = {default = 24, min = 1, max = 100},
})

local meter = k.object_2d("Meter", {
    components = {progress},
})

function meter:on_bounds()
    return k.rect(-self.width / 2, -self.height / 2, self.width, self.height)
end

function meter:on_draw(ctx)
    local x, y = -self.width / 2, -self.height / 2
    ctx:rect(x, y, self.width, self.height, k.color("#242424"))
    ctx:rect(x, y, self.width * self.value, self.height, self.fill)
end

function meter:advance(amount)
    return self:by {value = amount}
end

return function()
    local first = meter({
        id = "first",
        fill = k.color("#008CFF"),
        position = k.vec2(0, -60),
    })
    local second = meter({
        id = "second",
        fill = k.color("#FFFFFF"),
        position = k.vec2(0, 60),
    })
    k.world_2d():add(first)
    k.world_2d():add(second)
    k.all(function()
        first:advance(0.8):play(2)
        second:advance(0.5):play(2)
    end)
    k.wait(0.7)

    k.all(function()
        first:reset():play(1)
        second:reset():play(1)
    end)
    k.wait(0.5)
end
