return function()
	local ball = k.circle{ radius = 32 }
	k.world_2d():add(ball)

	local signal = ball:signal(function(object, frame)
		object.position:set(k.vec2(math.cos(frame.time * 5), math.sin(frame.time * 5)) * 128)
	end)

	k.wait(2)
	signal:stop()
	k.wait(1)
end
