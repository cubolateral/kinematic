return function(s)
	local card = s:rect {
		size = k.vec2(256, 256),
		round = k.quad(32, 32, 32, 32),
		fill = k.color("#008CFF80"),
		stroke = k.color("#008CFF"),
		stroke_width = 16,
	}
	s:world_2d():add(card)

	card:to { blur = 16 }:play(1)
	card:to { blur = 0 }:play(1)

	s:wait(1)
end
