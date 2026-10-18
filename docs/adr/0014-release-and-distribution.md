# 14. Releases from version tags, Homebrew cask in an own tap

Date: 2026-10-17

## Status

Accepted

## Context

Mailtomb is MIT licensed and should be installable with Homebrew and downloadable from
GitHub. Since 2026-09-01 Homebrew no longer supports casks that fail Gatekeeper and has
removed `--no-quarantine`; an unsigned macOS app installed through a cask cannot be
opened without manual workarounds.

## Decision

- Pushing a tag `vX.Y.Z` runs `.github/workflows/release.yml`: it checks that the tag
  matches `package.json` and `Cargo.toml`, creates a draft release, builds macOS
  (aarch64 and x86_64), Linux and Windows with `tauri-action`, publishes the release and
  renders `packaging/homebrew/mailtomb.rb.template` into
  `mehmetmuejde/homebrew-tap` (`Casks/mailtomb.rb`).
- macOS builds are signed with a Developer ID certificate and notarized. The secrets
  `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`,
  `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` and `HOMEBREW_TAP_TOKEN` live in the
  repository settings.
- `pnpm release:version X.Y.Z` sets the version in all manifests; `tauri.conf.json`
  reads it from `package.json`.
- Every third-party action is pinned to a commit SHA.

## Consequences

- One tag produces every artifact; there is no manual upload.
- Without the Apple secrets the macOS build is unsigned and the cask is not usable.
- The tap repository has to exist before the first release.
