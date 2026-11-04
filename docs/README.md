# Kinematic documentation

This directory contains the VitePress site, its dependencies, scripts, tests, and
configuration. The GitHub Actions workflow lives at `.github/workflows/docs.yml`, where
GitHub requires it.

From the repository root, prepare the runtime as described in
[DEVELOPMENT.md](../DEVELOPMENT.md), then build the application:

```sh
cargo build --locked
```

Run the site commands from `docs/`:

```sh
cd docs
npm ci
npm run docs:dev
```

Open the address printed in the terminal. To generate previews and build a validated
static site, make sure FFmpeg and ffprobe are on your PATH and an OpenGL session is
available:

```sh
npm run docs:media
npm run docs:build
npm run docs:preview
```

`npm run docs:test` checks API generation and version publishing. `npm run
docs:examples` runs `--check` on each project. Previews, API pages, and caches are
generated locally and ignored by Git. Temporary files go in `.cache/`, and the static
site goes in `.vitepress/dist/`.

See the [contribution guide](guide/contributing.md) for API documentation, version
configuration, and GitHub Pages publishing.

Guide projects live in [`guide/projects/`](guide/projects/README.md), with one project
per page and one numbered scene per demonstration. Their manifests list scenes in
explanation order. Open a topic folder in Kinematic and edit its scenes directly.
These sources and their required assets are versioned. Documentation commands never
regenerate or overwrite them.

The guide includes each complete Lua scene from its project file. A short focused
excerpt can remain visible above the expandable complete source. Update that excerpt
when changing the demonstrated operation. Write whole numbers without a redundant
`.0` in Lua examples, such as `play(1)` or `play(1/2)`. Keep fractional values and
floating-point syntax required by other languages, such as GLSL and RON. Videos loop silently with native controls
and respect reduced motion.

For guide-only validation without exporting:

```sh
npm run docs:examples -- --guide-only
```

`npm run docs:media -- --guide-only` is a separate explicit render command. It selects
one scene at a time in a temporary copy under `.cache/media/`, leaving the topic
project and its scene list unchanged. Static `GuideDemo` blocks use `still`, have no
`wait`, and publish a PNG of their initial frame. Animated demos publish a video and
poster. See the contribution guide for markup and palette conventions.
