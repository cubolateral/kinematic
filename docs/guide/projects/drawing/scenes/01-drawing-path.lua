local badge = k.object_2d("Badge", {
	on_bounds = function(self)
		return k.rect(-205, -155, 410, 310)
	end,

	on_draw = function(self, ctx)
		local path = k.path()
		path:move_to(-200, 100)
		path:line_to(200, 100)
		path:quad_to(0, -150, -200, 100)
		path:close()

		ctx:fill(path, {
			from = { -200, 0 },
			to = { 200, 0 },
			colors = { k.color("#FF4040"), k.color("#008CFF") },
		})
		ctx:stroke(path, k.color("#FFFFFF"), 4)
	end,
})

return function()
	local object = badge({})
	k.world_2d():add(object)

	object:to { rotation = math.pi }:play(1)
	k.wait(1)
end
