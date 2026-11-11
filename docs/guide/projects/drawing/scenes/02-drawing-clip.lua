local clipped = k.object_2d("Clipped", {
	on_bounds = function(self)
		return k.rect(-160, -110, 320, 220)
	end,

	on_draw = function(self, ctx)
		local mask = k.path()
		mask:move_to(-100, -100)
		mask:line_to(100, -100)
		mask:line_to(0, 100)
		mask:close()

		ctx:save()
		ctx:clip(mask)
		ctx:rect(-150, -100, 300, 200, k.color("#FF4040"))
		ctx:translate(70, 0)
		ctx:circle(0, 0, 70, k.color("#008CFF"))
		ctx:restore()
		ctx:stroke(mask, k.color("#FFFFFF"), 3)
	end,
})

return function()
	k.world_2d():add(clipped({}))
end
