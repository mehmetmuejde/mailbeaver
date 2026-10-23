# 34. Development build separate from the installed app

Date: 2026-10-23

## Status

Accepted

## Context

`pnpm tauri dev` and the installed app shared the identifier, so they shared the
database, the notification identity and the single-instance lock (ADR-0033), and both
wanted ports 1026 and 8026. A development build could not run while the installed app
caught mail, and test mail from development ended up in the everyday inbox.

## Decision

- `src-tauri/tauri.dev.conf.json` sets the identifier
  `io.github.mehmetmuejde.mailbeaver.dev`; `pnpm tauri:dev` runs
  `tauri dev --config src-tauri/tauri.dev.conf.json`. Release builds use
  `tauri.conf.json` alone.
- Under `tauri::is_dev()` the desktop host starts from the default ports 1027 (SMTP) and
  8027 (HTTP). `MAILBEAVER_*` variables and ports saved in the app still win
  (ADR-0031). The core and its defaults stay unchanged.
- The dev icon will live in the same file once it exists.

## Consequences

- Development and installed app run side by side, each with its own data directory.
- Mail sent to 1026 during development reaches the installed app; applications under
  test point at 1027 to reach the development build.
- A plain `pnpm tauri dev` still runs with the release identifier and is not the
  documented way to start the app.
