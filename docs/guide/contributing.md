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
of the source projects. Videos and images are rendered from clean copies under `target/`
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
