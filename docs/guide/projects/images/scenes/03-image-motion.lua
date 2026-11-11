return function()
	local picture = k.image_2d{
		source = k.load_image("assets/kitten.jpg"),
		size = k.vec2(280, 310),
	}
	k.world_2d():add(picture)

	picture:to {
		size = k.vec2(280, 280),
		round = k.quad(140, 140, 140, 140),
		fill = k.color("#FFFFFF"),
	}:play(2)

	k.wait(1)
end
