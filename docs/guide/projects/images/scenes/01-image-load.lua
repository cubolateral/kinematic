return function()
	k.world_2d():add(k.image_2d{
		source = k.load_image("assets/kitten.jpg"),
		scale = k.vec2(0.6, 0.6),
	})
end
