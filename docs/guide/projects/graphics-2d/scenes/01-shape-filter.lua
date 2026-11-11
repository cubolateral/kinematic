return function()
	local card = k.panel{
		size = k.vec2(256, 256),
		round = k.quad(32, 32, 32, 32),
		fill = k.color("#008CFF80"),
		stroke = k.color("#008CFF"),
		stroke_width = 16,
	}
	k.world_2d():add(card)

	card:to { blur = 16 }:play(1)
	card:to { blur = 0 }:play(1)

	k.wait(1)
end
