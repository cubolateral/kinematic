return function()
	local source = k.load_image("assets/kitten.jpg")

	k.world_2d():add(k.image_2d{
		source = source,
		size = k.vec2(240, 266),
		position = k.vec2(-180, 0),
	})
	k.world_2d():add(k.image_2d{
		source = source,
		size = k.vec2(240, 160),
		position = k.vec2(180, 0),
	})
end
