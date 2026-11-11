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

## Migrations

None. Kinematic has not published its first release. Development projects and examples
use the current API directly, without compatibility aliases or automatic migrations.
The manifest's `version: 1` describes its file format, not an application release.
