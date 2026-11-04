return function(s)
	local source = k.image("assets/kitten.jpg")

	s:world_2d():add(s:image_2d {
		source = source,
		size = k.vec2(240, 266),
		position = k.vec2(-180, 0),
	})
	s:world_2d():add(s:image_2d {
		source = source,
		size = k.vec2(240, 160),
		position = k.vec2(180, 0),
	})
end
