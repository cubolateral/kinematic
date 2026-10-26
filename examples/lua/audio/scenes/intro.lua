local bounce = k.object {
	state = { y = -100, velocity = 0 },
	inputs = {},
	update = function(state, frame, inputs)
		state.velocity = state.velocity + 600 * frame.dt
		state.y = state.y + state.velocity * frame.dt
		if state.y >= 100 then
			state.y = 100
			state.velocity = -360
			inputs.hit:play { pan = 0.3 }
		end
	end,
	bounds = function()
		return { x = -20, y = -120, width = 40, height = 240 }
	end,
	draw = function(ctx, properties, state)
		ctx:circle(0, state.y, 18, properties.fill)
	end,
}

return function(s)
	local boink_source = k.audio_synth { preset = "boink" }
	local boink = s:audio { source = boink_source, gain = 0.2 }
	boink:play()
	boink:play { gain = 0.15, pan = -0.5 }
	s:wait(0.6)

	for _, preset in ipairs { "pluck", "click", "impact", "whoosh" } do
		s:audio { source = k.audio_synth { preset = preset } }:play()
		s:wait(0.6)
	end

	s:audio {
		gain = 0.15,
		pan = -0.5,
		source = k.audio_synth {
			waveform = "triangle", frequency = 800, end_frequency = 220,
			attack = 0.01, decay = 0.1,
			sustain = 0.4, hold = 0.05, release = 0.2, overtone = 0.1,
		},
	}:play()
	s:wait(0.6)

	local recording = k.audio_file("assets/chime.wav")
	local left = s:audio { source = recording, gain = 0.4, pan = -0.6 }
	local right = s:audio { source = recording, gain = 0.4, pan = 0.6 }
	left:play()
	s:wait(0.3)
	right:play { gain = 0.3 }
	s:wait(0.6)

	local ball = s:object(bounce, { id = "ball", fill = k.color("#FBBF24") })
	ball:inputs { hit = boink }
	s:world_2d():add(ball)
	s:wait(4)
end
