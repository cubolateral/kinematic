return function(s)
	local picture = s:image_2d {
		source = k.image("assets/kitten.jpg"),
		size = k.vec2(280, 310),
	}
	s:world_2d():add(picture)

	picture:to {
		size = k.vec2(280, 280),
		round = k.quad(140, 140, 140, 140),
		fill = k.color("#FFFFFF"),
	}:play(2)

	s:wait(1)
end
