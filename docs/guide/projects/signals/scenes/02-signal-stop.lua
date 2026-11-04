return function(s)
	local ball = s:circle { radius = 32 }
	s:world_2d():add(ball)

	local signal = ball:signal(function(object, frame)
		object.position:set(k.vec2(math.cos(frame.time * 5), math.sin(frame.time * 5)) * 128)
	end)

	s:wait(2)
	signal:stop()
	s:wait(1)
end
