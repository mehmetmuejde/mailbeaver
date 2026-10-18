# 28. Cargo workspace, headless server and Docker image

Date: 2026-10-17

## Status

Accepted

## Context

Mailbeaver ran only inside the Tauri desktop app. Teams want it next to their
application in Docker Compose or CI, where no desktop exists. ADR-0001 kept the core
free of Tauri for this; ADR-0013 named the headless binary as the trigger to split the
crate, since a binary for Linux containers must build without Tauri and its system
libraries (WebKitGTK).

## Decision

- A Cargo workspace at the repository root with three packages:
  - `crates/core` (`mailbeaver-core`): every feature slice and the shared
    infrastructure of ADR-0013, Tauri-free. Hosts use its public modules
    (`server`, `config`, `http::Services`, `messages`, …).
  - `src-tauri` (`mailbeaver`): the desktop host — tray, notifications, Tauri commands.
  - `crates/server` (`mailbeaver-server`): the headless host.
- Shared package metadata, exactly pinned dependencies, lints and the release profile
  live in the root `Cargo.toml`; one `Cargo.lock` and one `target/`.
- The headless server serves the built UI on the API port. `build.rs` embeds `dist/`
  with `include_bytes!`, so the binary is self-contained without an extra crate; the
  UI uses hash routing, so only the files themselves are served (CSP as in the desktop
  app). The UI talks to the same origin.
- Configuration stays environment variables; new ones: `MAILBEAVER_DATA_DIR`
  (default `./data`, `/data` in the image), `MAILBEAVER_ALLOWED_HOSTS` (extra `Host`
  names besides loopback), `MAILBEAVER_PUBLIC_SMTP_ADDRESS` and
  `MAILBEAVER_PUBLIC_HTTP_ADDRESS` (shown in the UI behind port mapping).
- Bound to a non-loopback address without `MAILBEAVER_ALLOWED_HOSTS`, the `Host` check
  is off and the server logs a warning. The Compose example publishes the ports on
  loopback and sets the allowed hosts. There is no login: Mailbeaver is a development
  tool.
- `GET /api/health` and `mailbeaver-server healthcheck` (for the image's
  `HEALTHCHECK`; the image has no shell or curl).
- Docker image: multi-stage (Node builds the UI, Rust builds the server), runtime
  `gcr.io/distroless/cc-debian12:nonroot`, base images pinned by digest, volume
  `/data`. CI builds it on every push; a release tag pushes
  `ghcr.io/mehmetmuejde/mailbeaver:<version>`, `<major>.<minor>` and `latest` for
  amd64 and arm64.

## Consequences

- `cargo` commands run from the repository root (`--workspace`); the Tauri CLI finds
  the workspace by itself.
- Items the hosts need are public API of the core, e.g. the id newtypes.
- The server needs `pnpm build` before `cargo build -p mailbeaver-server` to contain
  the UI; without it `/` explains that, and the API still works.
- Settings that only the desktop app uses (tray grouping, notifications) are present
  but have no effect in the headless server.
