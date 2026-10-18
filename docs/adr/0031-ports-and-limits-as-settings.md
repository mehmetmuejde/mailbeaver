# 31. Ports and limits as settings

Date: 2026-10-17

## Status

Accepted

## Context

Ports and limits were environment variables only (ADR-0005, ADR-0028). An app started
from Finder or installed with Homebrew gets no shell environment, so desktop users had
no way to change a port — not even after a port conflict, when Mailbeaver cannot start
its HTTP API at all.

## Decision

- The settings row keeps `smtp_port`, `http_port`, `max_messages` and `max_message_mb`
  (migration `0002_server_settings.sql`); `NULL` keeps the default.
- Precedence: **environment variable > saved setting > default**
  (`Config::from_env(…).with_saved(…)`, pure and tested). `/api/info` lists the
  settings an environment variable fixes (`fromEnvironment`); the UI shows them
  read-only with the variable's name. This keeps Docker setups predictable.
- Changes apply at the next start. `GET/PUT /api/server-settings` read and save them;
  Settings → General → "Ports and limits" offers "Restart now" in the desktop app
  (Tauri command `restart_app`). The headless server asks for a restart of the
  server or its container.
- After a port conflict the HTTP API is down, so the error page of the desktop app
  saves new ports through the Tauri command `save_server_ports` straight into the
  database and restarts.
- `/api/info` also names the host (`desktop` or `server`) so the UI offers only what
  that host can do.

## Consequences

- One more migration; the bind address, allowed hosts and public addresses stay
  environment variables (they matter for Docker, not for the desktop app).
- A saved port that is taken leads to the same error page, which can change it again.
