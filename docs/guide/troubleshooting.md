---
pageClass: guide-page
---

# Troubleshooting

## An image looks stretched

**Symptom:** a cat looks too wide or too tall after loading.

**Check:** compare the source width-to-height ratio with the Image object's `size`.

**Cause:** a custom size or unequal scale changes the source aspect ratio.

**Fix:** use `height = width * source_height / source_width`, then apply the same scale
to both axes. See [images](images.md).

## A signal jumps when it stops

**Symptom:** an object snaps back at `signal:stop()`.

**Check:** compare the last derived position with its underlying track value.

**Cause:** signal writes last only for the evaluated frame.

**Fix:** schedule a base value matching the signal's endpoint, or keep the relationship
active. See [temporary writes](signals.md#temporary-writes).

## A Code edit changes the wrong characters

**Symptom:** a replacement targets an unexpected part of the displayed source.

**Check:** inspect the 1-based start and exclusive end, and any edits before this one.

**Cause:** ranges store coordinates and do not follow changed text.

**Fix:** clear old emphasis and query again after editing. See [Code ranges](code.md#ranges-do-not-follow-edits).

## Random results change unexpectedly

**Symptom:** rebuilding or seeking produces a different arrangement or trajectory.

**Check:** seed, number and order of draws, and whether the generator lives in state.

**Cause:** changing the sequence changes later draws. A generator outside simulation
state does not participate in checkpoint reconstruction.

**Fix:** use stable ordered arrays, fork independent effects, and preserve the simulation
generator in state. See [random](random.md).

## An object is missing

Check that it is attached to an active world or group. Creating an object does not
activate it. Review its lifetime, opacity, dimensions, scale, position, and camera. In
3D, check the clipping planes and lighting, or try `unlit=true`. A texture needs a
canvas attached to the root and a dependency graph without cycles.

## An animation does not run

Call `:play(...)` after `to`, `by`, `from`, or a transition method. `play()` is instant,
so pass a duration to see motion. Operations in `parallel` and `loop` do not extend the
parent timeline. Add a wait when the parent needs more time. Avoid writing the same
property from ambiguous concurrent scopes.

## A change disappears during playback

Inspector edits are temporary and are not saved to Lua. Copy the value into your code.
Signal writes are also temporary for each frame. Use simulation state to accumulate
reproducible values.

## The code changed but the preview did not

Check the status in **Console** and read the diagnostics. A failed candidate does not
replace the last valid revision. Export also uses the applied revision. Resolve imports,
duplicate IDs, property types, missing assets, or callback errors before expecting the
image to change.

## An image, font, or audio file cannot load

Use paths relative to the project root and match filename capitalization. Do not use
absolute paths, `..`, or external symbolic links. Include assets and their licenses when
sharing a project. Text and Code use a build-time default font. For portable projects,
provide a project font as in the Text and Code examples. FFmpeg is included with the
distribution or development setup, so it does not need to be on PATH.

Image files are limited to 64 MiB and 16 million decoded pixels. Other common resources
also have a 64 MiB read limit. Encoded and decoded audio has no fixed size limit and
stays in memory, so long tracks can use a lot of RAM. A decoding error or timeout leaves
the previous revision active.

## A shader passes check but fails to render

`--check` does not compile GLSL. Image shaders include `#version 330 core`. Mesh shaders
omit the version directive. Check shader inputs, declared uniforms, and active use of
required matrices in custom vertex shaders. See the [GLSL contract](shaders.md).

## A simulation differs or runs slowly

Keep the random generator in `state`, declare inputs, and avoid relying on `pairs` order
for sums or sampling. Do not capture mutable external values in update or mutate
callbacks. Checkpoints are reused only while logic and dependencies remain compatible.
Changing FPS or inputs may require replay from the beginning.

## Limits and performance

Each scene VM has 64 MiB of memory and instruction deadlines. Full scene construction
and evaluation have a two-second limit. Drawing has a 100 ms limit. Replay yields
between steps after about 8 ms, but one callback can use its full deadline. These Lua
limits cannot interrupt native Rust or GPU work.

Project and canvas dimensions are limited to 8192, and FPS to 1000. Meshes support up to
one million vertices and three million indices. Audio supports up to 100,000 playbacks
per scene. Drawing and text submissions also have limits. Split large workloads instead
of relying on the maximum.

The VM does not provide `io`, `os`, `package`, `debug`, arbitrary loaders, or the
ability to change the working directory. Libraries that depend on these features will
not work.

Reuse immutable resources, keep bounds accurate, and avoid creating identical geometry
every frame. Instances reuse the mesh cache. Console reports construction, replay,
apply, GPU validation, geometry, and render timings. Upload times include CPU
construction and GL submission, but not asynchronous GPU completion.

## Compatibility and bug reports

Official builds target Windows and Linux x64 with OpenGL 3.3. Drivers and system fonts
can affect appearance, so include fonts in projects. Software Mesa validation does not
replace testing on real drivers.

To report a problem on [GitHub](https://github.com/cubolateral/kinematic/issues),
include the version, operating system, GPU and driver, Console error, steps to
reproduce, and a minimal project with licensed assets. Specify whether the issue occurs
in `--check`, shader rendering, or editing. See [versions and migrations](versions.md).

## External editor does not open

Check **Editor settings → External editor** and test both commands with a project open.
Use the installed executable, without quotes, and JSON arrays for arguments. Verify
that its launcher is available on PATH, or provide an absolute executable path. On
Windows, use `Code.exe` or `VSCodium.exe`. Errors appear in the test result or Console.
If completion is missing, install LuaLS support in the external editor and check the
configuration described in the [editor guide](editor.md#edit-and-diagnose-a-revision).
Save changes externally before checking the preview.
