return function(s)
	local ball = s:circle {}

	s:wait(1)
	s:world_2d():add(ball)
	s:wait(1)
	ball:remove()
	s:wait(1)
end
