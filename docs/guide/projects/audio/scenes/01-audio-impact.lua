return function(s)
	local ball = s:circle {
		radius = 32,
		position = k.vec2(-200, 0),
	}
	s:world_2d():add(ball)

	local hit = s:audio {
		source = k.audio_synth { preset = "pluck", frequency = 520 },
		gain = 0.25,
	}

	ball.position.x:to(200):play(1, "linear")
	hit:play { pan = -0.7 }

	ball.position.x:to(-200):play(1, "linear")
	hit:play { pan = 0.7 }

	s:wait(1)
end
