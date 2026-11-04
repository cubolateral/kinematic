return function(s)
	local parent = s:group_2d {}
	s:world_2d():add(parent)

	for _, position in ipairs({ { 0, -128 }, { 128, 0 }, { 0, 128 }, { -128, 0 } }) do
		parent:add(s:circle {
			radius = 32,
			position = k.vec2(position[1], position[2]),
		})
	end

	parent:to { rotation = math.pi / 4 }:play(1)
	parent:to { position = { x = -256 } }:play(1)
	parent:to { position = { x = 256 }, rotation = -math.pi / 4 }:play(1)
	parent:to { position = { x = 0 }, rotation = 0 }:play(1)

	s:wait(1)
end
