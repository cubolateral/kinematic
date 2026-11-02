# Audio

## Audio resources and Lua players

An audio resource contains immutable samples. A player stores playback defaults for a
scene. Creating either one is silent. `player:play()` schedules a sound at the current
time without advancing the timeline.

```lua
local source = k.audio_synth {
    preset = "pluck",
    frequency = 520,
}

return function(s)
    local hit = s:audio {source = source, gain = 0.25, pan = 0}
    hit:play()
    s:wait(0.5)

    hit:play {pan = -0.7}
    s:wait(0.5)
end
```

`gain` ranges from 0 to 1, and `pan` ranges from -1 to 1. Overrides affect only that
playback. A player defaults to gain 0.2 and pan 0. Players and playbacks share the
resource's samples. `resource.duration` returns seconds. Sounds scheduled during scene
construction contribute to the scene duration.

## Audio files and synthesis

`k.audio_file("assets/hit.wav")` decodes WAV, MP3, OGG, or FLAC using the FFmpeg bundled
or prepared with the application. Paths are relative to the project root. File data is
frozen into the revision and converted to 48 kHz stereo PCM with short fades at the
edges. Audio stays in memory, so long tracks are not streamed.

`k.audio_synth` uses an attack, decay, sustain, hold, and release envelope. The `boink`,
`pluck`, `click`, `impact`, and `whoosh` presets are starting points for the same
synthesizer. You can change the waveform (`sine`, `triangle`, `noise`), starting and
ending frequency, noise, overtone, cutoff, and seed. The full envelope must last between
0.001 and 10 seconds.

See [AudioSynthOptions](../api/AudioSynthOptions.md) for options, defaults, and limits.
Create a new resource to change synthesis settings. `play` accepts only gain and pan.
The API does not provide pitch shifting, audio loops, reverb, continuous synthesis, or
spatial audio.

## Sync sound with motion

Schedule audio at the matching point in a visual sequence, as in the [audio
project](../examples/audio.md):

<<< ../../examples/lua/audio/scenes/intro.lua

For a simulated collision, declare the player as an input and call `inputs.hit:play
{gain=0.15}` from the update callback. Playback uses the fixed-step time. Replay
replaces occurrences by object, step, and update order, and checkpoints preserve that
history. Leave time at the end of the scene to hear the full sound. Drawing, signals,
and repeat cycles cannot schedule audio.

## Project audio clips

Clips imported through the editor belong to the project, independently of scenes and Lua
players. Their times are absolute within the project and can extend its duration. Lua
cannot edit these clips. The manifest stores them like this:

```ron
audio: [
    (id: 1, lane: 0, file: "assets/hit.wav", at: 1.0,
     offset: 0.0, duration: 0.2, gain: 0.5, pan: -0.5),
],
```

`at` is the start time in the project, `offset` is the start within the file, and
`duration` is the clip length in seconds. IDs must be unique. `lane` controls only the
vertical position. Both kinds of audio use the same mixer. Avoid scheduling the same
sound in Lua and importing it as a clip unless you intend to hear it twice.

## Edit and monitor audio

Drag audio from **Files** to the **Audio** lane in **Timeline**, or import it from the
audio area. Move or trim clips, split them with `Ctrl+B`, and delete them with Delete or
Backspace. Edits snap to frames. Clips in the same lane limit horizontal movement. Drag
vertically to separate overlapping clips into different lanes. Right-click for more
actions, or press Escape to cancel a drag. Original files are unchanged.

Use **Sound** to select overlapping clips and adjust volume and pan. The waveform
reflects the visible portion at the current zoom. Changes stay in memory until you press
`Ctrl+S`. Saving checks for external edits.

The speaker control beside playback adjusts preview volume from 0 to 100%. Its setting
is restored when you reopen the editor and does not affect exported audio. Pausing,
seeking, looping, or rebuilding clears queued audio. Preview latency includes one editor
frame and the SDL buffer.

During export, the mixer generates temporary PCM and muxes AAC into the MP4 without
re-encoding the video. Allow about 384 kB per second for stereo PCM, in addition to
temporary video storage. Sounds stop at scene boundaries, so leave enough time for their
release. See [export](export.md).
