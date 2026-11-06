# Kinematic documentation

This directory contains the VitePress site, its dependencies, scripts, tests, and
configuration. The GitHub Actions workflow lives at `.github/workflows/docs.yml`, where
GitHub requires it.

The generated API, Lua definitions, preview videos, posters, and static assets are
versioned in `api/` and `public/`. Viewing or building the site needs only Node.js 22
or later and Python 3.12:

```sh
cd docs
npm ci
npm run docs:dev
```

After changing the Lua API, renderer, or example projects, update the generated files
locally before committing:

```sh
npm run docs:update
npm run docs:test
npm run docs:build
```

`docs:update` prepares the runtime, builds the application with Cargo, validates the
example projects, generates the API, and exports videos and posters. It needs the
native dependencies in [DEVELOPMENT.md](../DEVELOPMENT.md), PowerShell 7, FFmpeg and
ffprobe on PATH, and a working OpenGL session. Review and commit changes under
`api/` and `public/` together with their sources.

`docs:dev` and `docs:build` use the versioned files directly. The GitHub documentation
workflow tests and builds the static site without Cargo, FFmpeg, or rendering. Temporary
files go in `.cache/`, and the static site goes in `.vitepress/dist/`.

For individual updates, `docs:generate` regenerates the API using an already-built
binary, `docs:examples` validates projects, and `docs:media` exports previews.

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
