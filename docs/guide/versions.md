---
pageClass: guide-page
---

# Versions and migrations

## Identify the documentation version

The menu and footer identify development or the stable documentation. `/dev/` follows
the default branch. `/stable/` documents the latest published release, with API pages
and previews saved in its tag. The application version is set in
`workspace.package.version`. The manifest remains at `version: 1` while that format is
supported.

Each tag containing versioned documentation assets also has a copy under
`/versions/vX.Y.Z/`. Automation builds a complete artifact containing development, stable, and tagged versions. Publishing
development does not remove stable docs. API source links use the documented tag or
revision.

## Migrate a project

1. Match the documentation menu to the application version you are running.
2. Copy the project before opening it in a newer version, including its scene metadata.
3. Read the migrations below and update affected Lua calls.
4. Open the copy, resolve diagnostics, seek through each scene, and export a short check.
5. Compare timing, event pauses, fonts, and audio before adopting the migrated copy.

The manifest's `version: 1` describes its file format, not the application release.
Choose a tagged documentation copy when maintaining an older executable. Publication
and editorial-fix procedures are described under [contributing](contributing.md#publish-with-github-pages).

## Existing migrations

### Audio

The old `s:sound {...}` API was replaced with a resource and a player. Previously,
synthesis parameters and playback were combined in the sound declaration. Now create the
samples separately and schedule a player:

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
