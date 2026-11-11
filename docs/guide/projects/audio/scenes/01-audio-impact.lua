return function()
	local ball = k.world_2d():add(k.circle{
		radius = 32,
		position = k.vec2(-200, 0),
	})

	local hit = k.audio {
		source = k.audio_synth { preset = "pluck", frequency = 520 },
		gain = 0.25,
	}

	ball.position.x:to(200):play(1, "linear")
	hit:play { pan = -0.7 }

	ball.position.x:to(-200):play(1, "linear")
	hit:play { pan = 0.7 }

	k.wait(1)
end
