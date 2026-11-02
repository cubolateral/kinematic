# Versions and migrations

## Identify the documentation version

The menu and footer identify development or the stable documentation. `/dev/` follows
the default branch. `/stable/` documents the latest published release, with API pages
and projects generated from its tag. The application version is set in
`workspace.package.version`. The manifest remains at `version: 1` while that format is
supported.

Each compatible tag also has a copy under `/versions/vX.Y.Z/`. Automation builds a
complete artifact containing development, stable, and tagged versions. Publishing
development does not remove stable docs. API source links use the documented tag or
revision.

## Releases and editorial fixes

The release workflow validates Windows and Linux builds and creates a draft for a
Cargo-compatible `vX.Y.Z` tag. A maintainer reviews and publishes the draft manually.
The documentation workflow runs after publication and reads the stable tag. It does not
rely on a second workflow being triggered by a draft created with `GITHUB_TOKEN`.

`docs/editorial.json` maps a tag to the full SHA of a commit containing corrected
guides. Add the mapping and run the workflow manually. The fix can then be reproduced in
later publications, while versions without a mapping keep the guides from their own tag.
Runtime code, metadata, examples, and API sources still come from the stable tag. This
allows explanations and links to be corrected without releasing an executable or
presenting the development API as stable. Changes to the API or example behavior require
a new application release.

Before the first release with compatible documentation infrastructure, automation
publishes only the development site. Tags that predate `--lua-definitions` are not
presented as complete generated API references.

## Existing migrations

### Audio

The old `s:sound {...}` API was replaced with a resource and a player:

```lua
local source = k.audio_synth {
    preset = "boink",
    frequency = 450,
}

local player = s:audio {
    source = source,
    gain = 0.2,
    pan = 0,
}
player:play()
```

Synthesis settings belong to the resource. Gain and pan belong to a player or playback.
Old clips attached to scenes are migrated to absolute project time while preserving
their sections and unique IDs. Save the project and check sync after opening an older
version.

### Project files and metadata

The current project uses `kinematic/project.ron`. Older layouts may migrate when opened
directly. The folder picker enables only folders with a valid manifest. Older event
metadata is migrated to `kinematic/scenes/`. Keep a backup before opening an older
project in a new version.

### Examples and documentation

The former small set of scene examples has been replaced by [examples organized by
feature](../examples/index.md). Regression fixtures used by tests live in
`src/tests/fixtures/`. New editor projects use built-in templates and do not depend on
example source files. The old `docs/lua.md` page now points to the concept guides and
generated API reference.

This page documents migrations present in the code. It does not announce future features
or compatibility with unreleased versions.
