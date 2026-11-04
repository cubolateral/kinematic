return function(s)
	local world = s:world_2d()

	local picture = s:image_2d {
		source = k.image("assets/cat-closeup.jpg"),
		size = k.vec2(320, 320),
		position = k.vec2(-180, 0),
	}
	world:add(picture)

	local cursor = s:circle {
		radius = 10,
		fill = k.color("#00000000"),
		stroke = k.color("#FFFFFF"),
		stroke_width = 3,
	}
	world:add(cursor)

	local swatch = s:rect {
		size = k.vec2(160, 160),
		position = k.vec2(180, 0),
	}
	world:add(swatch)

	local point = s:track(k.vec2(-120, -80))

	cursor:signal(function(object)
		local p = point:get()
		object.position:set(k.vec2(p.x - 180, p.y))
	end)
	swatch:signal(function(object)
		object.fill:set(picture:get_pixel_color(point:get()))
	end)

	point:to(k.vec2(120, 80)):play(2)
	point:to(k.vec2(-120, -80)):play(2)
end
