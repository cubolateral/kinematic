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
