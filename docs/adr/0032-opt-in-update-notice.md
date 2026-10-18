# 32. Opt-in update notice

Date: 2026-10-17

## Status

Accepted

## Context

Users of the downloaded app do not learn about new releases. Homebrew users can run
`brew upgrade`, but only if they know there is something to upgrade. An automatic
updater (`tauri-plugin-updater`) would need its own signing keys and would fight with
Homebrew, which also replaces the app.

## Decision

- A setting `update_check` (migration `0003_update_check.sql`), **off by default**:
  asking GitHub reveals the user's IP address.
- When on, the core asks
  `api.github.com/repos/mehmetmuejde/mailbeaver/releases/latest` right away and then
  once a day — also after a failed attempt, so no request storm can happen. The
  repository comes from `Cargo.toml`. The request uses the existing `reqwest`
  client; versions are compared by a small `major.minor.patch` parser (a pre-release
  is older than its release), no extra crate.
- `GET /api/update` reports current and latest version, the release page and whether
  an update is available. The sidebar links "Version x.y.z available" below the
  version; the settings mention `brew upgrade --cask mailbeaver`.
- Nothing is downloaded or installed.

## Consequences

- Until the repository has a published release, the check finds nothing (logged as
  info).
- The headless server shows the same notice; updating means pulling a new image.
