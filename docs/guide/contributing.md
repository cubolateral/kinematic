---
pageClass: guide-page
---

# Contributing and documentation

The source code is the authority for features and signatures. Read
[AGENTS.md](https://github.com/cubolateral/kinematic/blob/main/AGENTS.md) for
architecture contracts and
[DEVELOPMENT.md](https://github.com/cubolateral/kinematic/blob/main/DEVELOPMENT.md) for
native dependencies and distribution.

## Run the documentation site locally

Build the Rust application from the repository root, then run npm commands from `docs/`.
Install Node.js 22 or later, Python 3.12, and FFmpeg/ffprobe on PATH to generate and
check preview videos and posters. Prepare the runtime as described in DEVELOPMENT and
build the binary before generating the API:

```sh
cargo build --locked
cd docs
npm ci
npm run docs:dev
```

The site extends the VitePress theme with Kinematic's colors, corner radii, borders,
and control states from `packages/kinematic-ui/src/theme.rs`. Shared styles and the
JetBrains Mono font are configured in `docs/.vitepress/theme/`. Navigation, layout,
search, and syntax highlighting use VitePress components. Font licenses are in
`assets/fonts/`. `docs:dev`
generates the API before starting the server. If Rust metadata or Lua annotations
change, rebuild the binary and run `npm run docs:generate` again.

```sh
npm run docs:test

npm run docs:examples
npm run docs:media

npm run docs:build
npm run docs:preview
```

`docs:media` needs OpenGL and audio configured. `docs:build` checks internal links,
anchors, and assets. Generate media before the build because example pages use real
videos and posters.

Environment variables: `KINEMATIC_BIN` selects a built binary, `DOCS_BASE` changes the
base path (default `/kinematic/`), `DOCS_VERSION` identifies the version, and `DOCS_REF`
pins source links to a tag or revision. Scripts use paths relative to the checkout and
do not modify user exports.

## Keep the API in sync

The `--lua-definitions` CLI command exposes `scripting::definitions()`. The editor help
uses the same function. It reads the native object registry, macros, Rust `///`
comments, real properties, defaults, limits, and enum choices. The Markdown generator
combines these annotations with `src/scripting/api.lua` and `property.lua`. It does not
use rustdoc.

Document objects, fields, and methods next to their declarations. Update Lua annotations
for shared operations. Do not edit `docs/api/` or `docs/public/`: they are generated and
ignored by Git. Avoid maintaining duplicate property or default lists, and do not expose
Rust types for documentation.

Tests check function and field presence, method isolation by type, and reproducible
output files. When adding a method, update the runtime, metadata, and help together.
Short concept snippets can live in guides. Complete project examples should include the
original Lua files.

## Examples and validation

Each project under `examples/lua/` contains a manifest, scene files, and only the assets
it needs. The generator discovers manifests. Keep generated caches and output files out
of the source projects. Videos and images are rendered from clean copies under `docs/.cache/media/`
and stay out of Git.

Review each example's opening, development, and ending, along with readability, motion,
audio, and shaders. `--check` does not replace watching the render. For Rust changes,
run `cargo fmt --check`, `cargo check --workspace --locked`, and `cargo test --workspace
--locked`.

Reports in `docs/internal/` preserve historical measurements. Keep them separate from
the current user guides. The site structure takes inspiration from the concept
progression in [Motion Canvas](https://motioncanvas.io/docs/) and the separation of
tutorials and reference in [Godot](https://docs.godotengine.org/en/stable/), adapted to
Kinematic's features.

## Publish with GitHub Pages

In **Settings → Pages**, select **GitHub Actions** as the source. The documentation
workflow validates pull requests, publishes development at `/kinematic/dev/`, and builds
stable docs from the published release tag at `/kinematic/stable/`. The root points to
stable when a release is available, and to development before the first release.

The existing release workflow continues to create drafts after validating executables.
Publishing the draft starts the stable documentation build, which checks out the
matching tag. The workflow does not depend on an event created with `GITHUB_TOKEN` to
continue.

To republish an editorial fix, run **Documentation** manually. Add the tag and full
editorial commit SHA to `docs/editorial.json`, for example a `vX.Y.Z` key with a
40-character SHA, then run the workflow. The site continues to identify the application
tag. API pages, examples, and previews come from that tag, while other versions keep
their own guides. See [versions](versions.md) for the full contract.

## Write a practical guide section

Introduce a concrete problem, show the smallest runnable scene, attach its result above
the code, then explain why it looks that way. Finish with one parameter to change and
its expected effect. Use still frames for composition and controlled videos for motion.
Videos must have controls, a poster, an accessible label, and a caption. They loop
silently by default. The reader can pause, seek, or unmute. Respect the reduced-motion
preference by leaving playback paused. Capture editor screenshots from the real interface.

Keep editable guide projects in `docs/guide/projects/<page>/`. Each guide page has
one project with numbered Lua scenes, one per demonstration, listed in explanation
order in `kinematic/project.ron`. Edit these files directly in Kinematic. Validation
and rendering never regenerate them.

Reference a complete scene using its matching preview name and a VitePress include:

```md
<GuideDemo name="first-circle" caption="A white circle." still>

<!-- demo: first-circle -->
<<< ./projects/start/scenes/01-first-circle.lua

</GuideDemo>
```

A focused code block may stay visible with the full include in a `<template #source>`
slot. Keep blank lines around fences and includes. If the operation changes, update
its focused excerpt too. The preview name must match the `demo` marker, and its scene
must be listed in that page's project manifest.

Static scenes have no `wait` and publish their initial frame as a PNG. Use 960 × 540
framing at 30 FPS. Write whole Lua numbers without `.0`, including durations such as
`1` or `2`; fractions such as `1/2` are also valid. Prefer short durations, except deliberate
longer loops. Omit final waits when an animation closes a continuous loop.

Use `npm run docs:examples -- --guide-only` to validate the source projects without
exporting. When ready, `npm run docs:media -- --guide-only` renders one scene at a time
from clean temporary copies under `docs/.cache/media/`. It changes the scene list only
in those copies. Generated previews stay under `docs/public/previews/` and outside Git.

Keep required assets inside the topic project. Images uses
`docs/guide/projects/images/assets/` for both scene resources and downloadable photos.
Guide scenes must work with the documented runtime version. Review code, media,
caption, and version together before publishing.

Start with white objects. Use accent blue sparingly for secondary emphasis. For
comparisons, choose a few colors from `#FF4040`, `#FFBF40`, `#BFFF40`, `#40FF80`,
`#40FFFF`, `#4080FF`, `#BF40FF`, and `#FF40BF`. A secondary panel or track can use
`k.color("#0A0A0A")`, equivalent to RGBA `(10, 10, 10, 255)`. Numeric `k.color` arguments
use normalized channels, so do not pass those byte values directly. Separate Lua setup,
attachment, and scheduling where their purpose changes. Keep related operations together.

## Maintain the shared documentation colors

The default editor background is RGB `(10, 10, 10)` with full opacity, contrast is `0.5`,
and the shared corner radius is `4 px`. UI surfaces, hover and active states, borders,
and disabled text are derived from the background. The docs site uses this same
calculation, so changing its `BACKGROUND` and `CONTRAST` in
`docs/.vitepress/theme/index.ts` updates all derived colors. Kinematic linearizes each sRGB
background channel (`c / 12.92` when `c <= 0.04045`, otherwise
`((c + 0.055) / 1.055)^2.4`) and computes luminance as
`0.2126 R + 0.7152 G + 0.0722 B`. Luminance above `0.179` selects black foreground.
Otherwise it selects white. For each derived color, Kinematic mixes each RGB channel
with that foreground using `background + (foreground - background) * amount * contrast`.
The amount is `0.08` for surfaces, `0.14` for hover, `0.22` for active states, `0.12`
for borders, and `0.50` for disabled text. Alpha stays equal to the background alpha.

The code editor defaults to the One Dark syntax palette, with its background set to the
same RGB `(10, 10, 10)` value. Its syntax colors remain those of One Dark.
