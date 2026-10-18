# 1. Tauri desktop host with a headless-capable core

Date: 2026-10-17

## Status

Accepted

## Context

Mailcatcher catches outgoing mail during development. Developers want it as a desktop
app that lives in the tray, but teams also want to run it in CI or a container later.
Maintaining two separate implementations of the SMTP server, storage and API would
double the work and let them drift apart.

## Decision

The SMTP server, storage, mailbox logic and HTTP API live in one Rust core
(`server::start`) that does not depend on Tauri. Tauri 2 is only a host: it starts the
core inside its process, shows the window and the tray icon. A headless host (and a
Docker image) will start the same core without Tauri later.

## Consequences

- One implementation of every rule, shared by desktop and headless use.
- Tauri types must stay out of the core modules; only `lib.rs` and `tray.rs` use Tauri.
- The headless binary and the Docker image are still to be built.
