<div align="center">

# Kinematic

An animation editor for 2D and 3D scenes, with a live preview,
a visual timeline and audio in the same workspace.

[Downloads](https://github.com/cubolateral/kinematic/releases) ·
[Lua API](docs/lua.md) ·
[Examples](examples/lua) ·
[Development](DEVELOPMENT.md)

</div>

---

Scenes and animations are written in Lua. The editor provides code diagnostics,
a preview that updates as you edit, and a timeline for playback and audio.
Animations can be exported to MP4 with audio.

## Features

- **2D and 3D scenes.** Shapes, text, code, hierarchical canvases, cameras,
  projections, custom drawing and shaders.
- **Animation in Lua.** Property tweens, easing, sequences, parallel tasks and
  seekable simulations.
- **Editing and preview.** Code diagnostics, live preview, a shared
  selection across the scene tree and timeline, and an inspector for adjustments.
- **Audio.** Procedural audio and imported WAV, MP3, OGG or FLAC files,
  with waveform previews, trimming, splitting, volume and pan.
- **Export.** MP4 video with audio and screenshots. Editor
  overlays stay out of the result.

## Scene example

This scene moves a circle horizontally over two seconds, then waits for half
a second.

```lua
return function(s)
    local ball = s:circle {
        radius = 24,
        position = k.vec2(-100, 0),
        fill = k.color("#60A5FA"),
    }
    s:world_2d():add(ball)

    ball:to { position = k.vec2(100, 0) }:play(1)

    s:wait(1)
end
```

More examples cover [2D](examples/lua/2d), [3D](examples/lua/3d),
[audio](examples/lua/audio) and [simulations](examples/lua/simulation).

## Get started

Distributions target **Windows and Linux x64**, with one executable per platform.
Get available builds from [GitHub Releases](https://github.com/cubolateral/kinematic/releases).
LuaLS and FFmpeg are included; no separate installation or PATH setup is needed.
On Linux, give the downloaded file permission to run with `chmod +x`.

Open Kinematic, choose **Create project** or **Open project**, and edit a scene
in Code. Preview the animation, scrub the timeline and use Export to save it.

A desktop session with OpenGL 3.3 and an audio driver is required. Distribution
targets are Windows 10 22H2 or newer and Linux with glibc 2.35 or newer, using
the system's graphics, windowing, font and audio libraries.

## Documentation

- [Lua projects and API](docs/lua.md): scenes, objects, animation, audio and shaders.
- [Example projects](examples/lua): runnable scenes to explore and adapt.
- [Development guide](DEVELOPMENT.md): setup, tests, distribution and releases.

## License

[MIT](LICENSE). Bundled tools retain their own licenses.
