return function()
	local parent = k.world_2d():add(k.group_2d{})

	for _, position in ipairs({ { 0, -128 }, { 128, 0 }, { 0, 128 }, { -128, 0 } }) do
		parent:add(k.circle{
			radius = 32,
			position = k.vec2(position[1], position[2]),
		})
	end

	parent:to { rotation = math.pi / 4 }:play(1)
	parent:to { position = { x = -256 } }:play(1)
	parent:to { position = { x = 256 }, rotation = -math.pi / 4 }:play(1)
	parent:to { position = { x = 0 }, rotation = 0 }:play(1)

	k.wait(1)
end
