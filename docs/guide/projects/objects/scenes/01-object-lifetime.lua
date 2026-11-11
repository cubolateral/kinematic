return function()
	local ball = k.circle{}

	k.wait(1)
	k.world_2d():add(ball)
	k.wait(1)
	ball:remove()
	k.wait(1)
end
