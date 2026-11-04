return function(s)
	local leader = s:circle {
		radius = 32,
		position = k.vec2(-256, -64),
	}
	s:world_2d():add(leader)

	local follower = s:circle {
		radius = 32,
		fill = k.color("#008CFF")
	}
	s:world_2d():add(follower)

	follower:signal(function(object)
		local position = leader.position:get()
		object.position:set(k.vec2(position.x, position.y + 128))
	end)

	leader.position.x:to(256):play(2)
	s:wait(1)
end
