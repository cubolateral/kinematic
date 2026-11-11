local cells = {}

for y = 1, 12 do
    cells[y] = {}

    for x = 1, 20 do
        cells[y][x] = false
    end
end

cells[5][9], cells[5][10], cells[5][11] = true, true, true

local life = k.object_2d("Life", {
    state = {cells = cells, tick = 0}, inputs = {},
    on_update = function(self, frame)
        self.state.tick = self.state.tick + 1

        if self.state.tick % 6 ~= 0 then
            return
        end

        local next = {}

        for y = 1, 12 do
            next[y] = {}

            for x = 1, 20 do
                local count = 0

                for dy = -1, 1 do
                    for dx = -1, 1 do
                        if dx ~= 0 or dy ~= 0 then
                            local row = self.state.cells[(y + dy - 1) % 12 + 1]

                            if row[(x + dx - 1) % 20 + 1] then
                                count = count + 1
                            end
                        end
                    end
                end

                next[y][x] = count == 3 or count == 2 and self.state.cells[y][x]
            end
        end

        self.state.cells = next
    end,
    on_bounds = function(self)
        return k.rect(-200, -120, 400, 240)
    end,
    on_draw = function(self, ctx)
        for y, row in ipairs(self.state.cells) do
            for x, alive in ipairs(row) do
                if alive then
                    ctx:rect(-220+x*20, -140+y*20, 18, 18, self.fill)
                end
            end
        end
    end,
})

return function()
    local grid = life({id = "life", fill = k.color("#34D399")})

    k.world_2d():add(grid)

    k.wait(3)
end
