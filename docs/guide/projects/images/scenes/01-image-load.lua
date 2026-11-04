return function(s)
	s:world_2d():add(s:image_2d {
		source = k.image("assets/kitten.jpg"),
		scale = k.vec2(0.6, 0.6),
	})
end
