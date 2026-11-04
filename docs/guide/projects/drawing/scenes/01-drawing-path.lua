local badge = k.object {
	bounds = function()
		return { x = -205, y = -155, width = 410, height = 310 }
	end,

	draw = function(ctx)
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
}

return function(s)
	local object = s:object(badge, {})
	s:world_2d():add(object)

	object:to { rotation = math.pi }:play(1)
	s:wait(1)
end
