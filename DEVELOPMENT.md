# Developing Kinematic

This guide covers local development, validation, distribution and releases.
For scene authoring, see [Lua projects and API](docs/guide/start.md). Read
[AGENTS.md](AGENTS.md) for architecture boundaries and contributor conventions.

## Prerequisites

Development and distribution target Windows and Linux x64. Run commands from
this repository's root. Install stable Rust with rustfmt, Git, CMake, curl, tar
and a native C/C++ compiler. The editor and export validation need a desktop
session with OpenGL 3.3 and an audio driver.

### Windows

Install Visual Studio 2022 Build Tools or Visual Studio with the **Desktop
development with C++** workload and a Windows SDK. Use the MSVC Rust toolchain.
Windows PowerShell 5.1 is sufficient for the preparation and distribution script.
PowerShell 7 also works.

Distribution builds additionally require LLVM/Clang and Python 3. Skia has no
precompiled binary for the requested static Windows C runtime configuration, so
that build compiles Skia from source. Set `LLVM_HOME` to the LLVM installation
if it is not found automatically, and make Python available on PATH. Cargo
reuses the compiled result on subsequent builds. Ordinary development builds
use Skia's precompiled binaries.

### Linux

Use PowerShell 7 (`pwsh`) to run the same script as Windows. Install it
separately if it is not available in your distribution's package repositories.
The CI baseline is Ubuntu 22.04 x64. On Ubuntu, install the native dependencies:

```sh
sudo apt-get update
sudo apt-get install -y \
  build-essential cmake ninja-build pkg-config curl tar python3 \
  libfontconfig1-dev libfreetype6-dev libasound2-dev libpulse-dev \
  libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxi-dev \
  libxfixes-dev libxss-dev libxtst-dev libwayland-dev libxkbcommon-dev \
  libegl1-mesa-dev libgl1-mesa-dev xvfb
```

Xvfb is only needed for graphics validation without a physical display. The
[CI workflow](.github/workflows/ci.yml) records the build and graphics setup used
by the runners. Other distributions need equivalent packages.

## Prepare FFmpeg

On Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/distribute.ps1 prepare
```

On Linux:

```sh
pwsh -File scripts/distribute.ps1 prepare
```

Preparation downloads the archives pinned in
[scripts/dependencies.json](scripts/dependencies.json), verifies their SHA-256
hashes, extracts the tools and creates a compressed `runtime.tar.gz` under
`target/runtime/<platform>/`. Licenses and upstream documentation are preserved.
All downloads and generated files stay under the Git-ignored `target/` directory.

Run preparation once after cloning and again after changing the dependency
manifest or packaging script, or after `cargo clean`. Unchanged preparation is
reused, and verified downloaded archives are reused when rebuilding the bundle.
There are no FFmpeg downloads in `build.rs` or during application startup.

## Run and edit

After preparation, use ordinary Cargo commands:

```sh
cargo run
cargo run -- examples/lua/2d
cargo run -- examples/lua/3d
cargo run -- examples/lua/audio
```

Development builds run the extracted tools directly from `target/runtime/`.
They do not embed the compressed archive on every rebuild. SDL3, Lua and Skia
are linked statically. Distribution builds also link the Windows C runtime
statically. System graphics, windowing, font and audio libraries remain supplied
by the operating system. Rust dependencies are pinned by `Cargo.lock`.

Lua editing uses VS Code, VSCodium, or a custom graphical editor. Install LuaLS support
there and configure the executable in Editor settings. Kinematic generates `.luarc.json`
for new configurations and the API and custom-component support in
`kinematic/cache/luals/`. Existing configurations are preserved; merge the generated
`luarc.json` entries manually. See the [editor guide](docs/guide/editor.md).

The binary is the only application interface. Projects and scenes are written
in Lua. Rust modules and procedural macros are internal implementation details.
Use the existing UI components and architecture described in `AGENTS.md`.

### Command-line operations

Examples use paths relative to the checkout and work on both platforms:

```sh
cargo run -- --new target/my-animation "My animation"
cargo run -- --new target/my-3d-animation "My 3D animation" --3d
cargo run -- --check examples/lua/simulation
cargo run -- --render examples/lua/3d
```

`--check` constructs and evaluates scenes without opening a window. It does not
validate GPU drawing or shader compilation. `--render` uses the application's
existing graphics resources and exports to the project's
`output/<project name>.mp4`. Existing exports are preserved with numeric suffixes.
Use `cargo run -- --help` for the command summary.

## Validate changes

For Rust changes, run:

```sh
cargo fmt --check
cargo check --workspace --locked
cargo test --workspace --locked
```

Tests requiring additional graphics or audio setup are marked as ignored. Passing the ordinary
suite does not replace distribution validation. See
[graphics validation](docs/internal/lua-validation.md) and
[performance measurements](docs/internal/lua-performance.md) for those checks.

Pushes and pull requests run formatting, checking and tests on both Windows and
Linux. Tag builds also generate and validate the distributions.

## Build a distribution

Use the same command locally and in GitHub Actions.

On Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/distribute.ps1 dist
```

On Linux:

```sh
pwsh -File scripts/distribute.ps1 dist
```

`dist` prepares dependencies if needed, then performs a locked release build
with the `bundled` feature for the platform's explicit x64 target. It embeds the
compressed tools and writes the executable and SHA-256 checksum to `target/dist/`:

```text
kinematic-X.Y.Z-windows-x64.exe
kinematic-X.Y.Z-windows-x64.exe.sha256
kinematic-X.Y.Z-linux-x64
kinematic-X.Y.Z-linux-x64.sha256
```

Each invocation builds only the host platform's distribution. On Linux, the
recipient may need to run `chmod +x kinematic-X.Y.Z-linux-x64` after downloading.
The distribution targets are Windows 10 22H2 or newer and Linux with glibc 2.35
or newer, with the system desktop libraries and OpenGL 3.3 driver installed.

### What distribution validation checks

The script inspects native imports with `dumpbin` on Windows or `ldd` on Linux.
It rejects dynamic application libraries, unresolved Linux libraries and
Windows imports of the Visual C++ runtime.

It then copies only the executable to an isolated directory under
`target/runtime/<platform>/`, clears PATH and redirects
user data to that directory. Running `--self-test` checks:

- FFmpeg generates a tone and the application's audio decoder reads it.
- The SDL audio device consumes the decoded samples.
- The application renderer exports an MP4 with video and audio streams.
- FFmpeg decodes the exported streams and the exported audio is not silent.

The script runs the test twice, first with an empty dependency cache and then
with a warm cache. Failed runs retain their files for inspection. A successful
`dist` command never creates or publishes a GitHub Release.

For Linux validation without a display or audio device:

```sh
SDL_AUDIODRIVER=dummy LIBGL_ALWAYS_SOFTWARE=1 \
  xvfb-run -a pwsh -File scripts/distribute.ps1 dist
```

CI uses Xvfb/Mesa on Linux, a pinned Mesa software OpenGL build on Windows, and
SDL's dummy audio driver on both. The dummy driver checks the playback pipeline.
listen to playback on a real audio device before publishing.

### Runtime dependency cache

The distributed executable extracts FFmpeg lazily into
`.kinematic/cache/dependencies/` under the platform's user data directory.
FFmpeg runs as a separate process, using an explicit executable path.
The recipient does not need Rust or FFmpeg installed. Lua editing requires an external
graphical editor, with LuaLS support installed in that editor.

The cache key combines the platform and hashes of the pinned dependency
manifest and packaging script. It is independent of the editor version, so
editor versions with the same dependency packaging reuse the installation.
A file lock, staging directory, completion marker and atomic rename protect
against concurrent launches and interrupted extraction. Deleting the cache is
safe. The executable rebuilds it on demand without a network connection.

### Update bundled tools

Update the URLs and verified SHA-256 hashes together in
`scripts/dependencies.json`. Rerun `prepare`, the tests and `dist` on both
platforms. Preserve licenses and upstream documentation in the bundle.

The FFmpeg GPL build includes libx264. Preserve its license and corresponding
source availability when redistributing. Its monthly upstream assets have a
two-year retention period. Update or mirror the exact assets before expiry.
Upstream sources: [FFmpeg build recipes and retention policy](https://github.com/BtbN/FFmpeg-Builds).

## Publish a version

1. Update `workspace.package.version` in `Cargo.toml` to `X.Y.Z`.
2. Run `cargo check --workspace` to update `Cargo.lock`.
3. Run the validation commands and `dist` on Windows and Linux. Check actual
   playback and inspect the exported video.
4. Commit the version change and push the commit, then create and push its tag:

   ```sh
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

Tag builds reject any mismatch between `vX.Y.Z` and the workspace version.
Only after both platform jobs pass does Actions create a **draft** GitHub Release
with the executables and checksums. Review the draft, downloads, release notes
and dependency licenses, then publish it manually. Publication is never automatic.

## Editor workflow

Kinematic restores the screen used when it last closed: an open project is
reopened, while closing from home starts on home again. If a project is
unavailable, startup falls back to home. The home screen lists recent projects, searches
by name and opens projects with a double click. Right-click a project to open,
rename, duplicate, remove it from the list, copy its path or open a terminal.
Removing a list entry keeps its files. Unavailable projects keep their saved
names and appear dimmed with a strikethrough. Each row shows the project format
version and its last opening time in the local timezone.

Use **Create project** to choose a project path, or **Open project** to open an
existing directory. The default name is **Project**. Browse selects a parent
folder and appends the project name in snake case. The last selected folder is
cached. The path stays empty if no folder is saved or it is unavailable.
Renaming updates the path until it is edited manually. Browsing again restores
automatic updates. A new project
contains an editable Lua example, an `assets/` directory and
`kinematic/project.ron`. Creation accepts any root directory without overwriting
existing project files, with resolution and FPS presets defaulting to 1280×720
at 60 FPS. Opening requires a valid `kinematic/project.ron`.
Its name can also be changed in
**Project → Project settings**.

Use **Project → Save project** (`Ctrl+S`) to save the project and open files.
The home icon at the start of the editor toolbar (`Ctrl+Shift+Q`) returns home.
The top-right close icon (`Ctrl+Q`) exits. The home screen also has an editor settings icon.
Errors appear in centered message popups.

When opening a project, the folder browser enables **Select Folder** only for
directories containing a valid `kinematic/project.ron`. Older project layouts migrate when opened directly.
Change the scene in Code. The preview updates after a short pause. Save persists
the source files. Closing unsaved buffers or exiting offers Save, Discard and
Cancel.

The manifest lists scenes in playback order and stores resolution, FPS and
margins. New projects default to 1280 x 720 at 60 FPS. Inspector edits are temporary and reset on playback, seek, or reload. Scene Tree, Timeline
and Preview share object selection. Failed reloads preserve the last valid preview.

Import WAV, MP3, OGG or FLAC files through the Timeline or drag them from Files.
Move, trim, and split clips without modifying their sources. Edits snap to
project frames. Sound controls clip volume and pan. The Export menu provides
video export and screenshots, excluding editor overlays. See
[Lua projects and API](docs/guide/start.md) for scene authoring and the full audio workflow.

## Storage

Project sources, assets, `kinematic/project.ron`, runtime metadata under `kinematic/`
and exports under `output/` belong to the project directory. Global settings,
preview volume, text editor zoom, recent project details and the last opened
project stay under `.kinematic` in the
platform's user data directory:

- Linux: `$XDG_DATA_HOME`, or `~/.local/share` when unset.
- macOS: `~/Library/Application Support`.
- Windows: `%APPDATA%`.

Window position, size and panel layout are not saved.

## Documentation

Website tooling, dependencies and development instructions live in
[docs/README.md](docs/README.md).
