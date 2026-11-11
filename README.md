<div align="center">

<img src="assets/brand/logo.png" alt="Kinematic logo" width="128">

# Kinematic

An animation editor for 2D and 3D scenes, with a live preview,
a visual timeline and audio in the same workspace.

[Downloads](https://github.com/cubolateral/kinematic/releases) ·
[Documentation](https://cubolateral.github.io/kinematic/) ·
[Examples](examples/lua) ·
[Development](DEVELOPMENT.md)

</div>

---

Scenes and animations are written in Lua. The editor provides runtime diagnostics,
a preview that updates after external saves, and a timeline for playback and audio.
Animations can be exported to MP4 with audio.

## Features

- **2D and 3D scenes.** Shapes, text, code, hierarchical canvases, cameras,
  projections, custom drawing and shaders.
- **Animation in Lua.** Property tweens, easing, sequences, parallel tasks and
  seekable simulations.
- **Editing and preview.** External Lua editing, live preview, a shared
  selection across the scene tree and timeline, and an inspector for adjustments.
- **Audio.** Procedural audio and imported WAV, MP3, OGG or FLAC files,
  with waveform previews, trimming, splitting, volume and pan.
- **Export.** MP4 video with audio and screenshots. Editor
  overlays stay out of the result.

## Scene example

This scene moves a circle horizontally over two seconds, then waits for half
a second.

```lua
return function()
    local ball = k.circle{
        radius = 24,
        position = k.vec2(-100, 0),
        fill = k.color("#60A5FA"),
    }
    k.world_2d():add(ball)

    ball:to { position = k.vec2(100, 0) }:play(2)

    k.wait(0.5)
end
```

More examples cover [2D](examples/lua/2d), [3D](examples/lua/3d),
[audio](examples/lua/audio) and [simulations](examples/lua/simulation). The [gallery](docs/examples/index.md) also covers
Code, LaTeX, signals, custom objects, cameras, procedural drawing and shaders.

## Get started

Distributions target **Windows and Linux x64**, with one executable per platform.
Get available builds from [GitHub Releases](https://github.com/cubolateral/kinematic/releases).
Lua and FFmpeg are included. Install VS Code, VSCodium, or a graphical editor and its LuaLS support for Lua editing.
On Linux, give the downloaded file permission to run with `chmod +x`.

Open Kinematic, choose **Create project** or **Open project**, and edit a scene
in your external editor. Preview the animation, scrub the timeline and use Export to save it.

A desktop session with OpenGL 3.3 and an audio driver is required. Distribution
targets are Windows 10 22H2 or newer and Linux with glibc 2.35 or newer, using
the system's graphics, windowing, font and audio libraries.

## Documentation

- [Documentation](https://cubolateral.github.io/kinematic/): guides, tutorials and generated Lua API.
- [Guide sources](docs/guide/start.md): documentation maintained in this repository.
- [Example projects](examples/lua): runnable scenes to explore and adapt.
- [Development guide](DEVELOPMENT.md): setup, tests, distribution and releases.

## License

[MIT](LICENSE). Bundled tools retain their own licenses.
