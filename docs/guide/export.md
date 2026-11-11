---
pageClass: guide-page
---

# Export and command line

## Review a first export

1. Set resolution and FPS in Project settings and save the project.
2. Wait for code changes to apply successfully. Resolve any diagnostics.
3. Choose **Export → Export** for the project or **Export scene** for the current scene.
4. Open the resulting MP4 in `output/` and check its first frame, motion, ending, and sound.

Try changing 960 × 540 to 1920 × 1080 while keeping scene coordinates unchanged. The
output has more pixels and a different visible canvas area at the same 2D camera zoom.
Recheck framing. Changing FPS changes temporal sampling and simulation steps. A
60 FPS export samples motion twice as often as a 30 FPS export of equal duration.

Editor margins, selection outlines, navigation cameras, and preview volume are inspection
settings. They do not become part of the exported frame or audio mix.

## Resolution, frame rate, and MP4

Resolution and frame rate come from `kinematic/project.ron` or **Project settings**. New
projects default to 1280 × 720 at 60 FPS. Changing FPS also changes the fixed simulation
step, so review the result again.

Choose **Export → Export** to render the project or **Export scene** to render the
current scene. The MP4 includes mixed audio and is created with FFmpeg. Files are saved
to `output/<project name>.mp4`. Existing files are kept, with a number added to the new
filename.

**Save screenshot** captures an image without editor overlays. Use screenshots to
inspect a frame and a video to review timing, continuity, and audio. The Export menu
shows progress and provides **Cancel** while rendering.

## Revision and timing

Export uses the latest successfully applied preview revision from saved files. Changes made after export starts remain pending until it
finishes or is cancelled. A saved file with errors does not replace the last valid preview.
Check the status and diagnostics in **Console** before exporting.

Temporary Inspector changes are discarded. Editor cameras, selection, and overlays are
not included in the video. The renderer reuses the application's graphics resources and
advances exactly `1/fps` per frame, regardless of machine speed.

Runtime or drawing errors stop the export and remove the incomplete file. You can cancel
and resume editing. A later export starts from the beginning.

## Exported audio

The 48 kHz stereo mixer is shared with the preview and includes soft clipping. Project
clips and Lua playbacks are mixed together. Preview monitor volume does not affect the
exported file. Audio boundaries align to whole samples.

The temporary video receives AAC audio through muxing, without re-encoding the video. In
addition to the video file, allow about 384 kB per second for temporary PCM. Sound tails
stop at scene boundaries, so leave enough time for releases to finish.

## Command line

| Command | Result |
| --- | --- |
| `kinematic PROJECT` | Opens a project folder in the editor. |
| `kinematic --new DIRECTORY NAME` | Creates a 2D project. |
| `kinematic --new DIRECTORY NAME --3d` | Creates a project from the 3D template. |
| `kinematic --check PROJECT` | Builds and evaluates scenes without opening a window. |
| `kinematic --render PROJECT` | Opens graphics resources and exports the project. |
| `kinematic --lua-definitions` | Prints the LuaLS definitions used by external editors and the site. |
| `kinematic --help` | Shows a summary of commands. |
| `kinematic --self-test` | Validates the distributed runtime for maintenance. |

Quote project names that contain spaces. Use the folder containing `kinematic/`, not the
metadata subfolder.

`--check` does not validate shaders, GPU drawing, readability, or perceived sync.
`--render` requires an OpenGL 3.3 graphical session, even from the command line. On
Linux CI, use Xvfb/Mesa and the dummy audio driver, as the documentation workflow does.
A dummy driver cannot validate what audio sounds like.

## Render the examples

From `docs/`, run `npm run docs:examples` for logical validation and `npm run
docs:media` to render clean copies of each project under `docs/.cache/media/`. Public
previews are saved in `docs/public/previews/`. Original projects and their exports are
preserved.
