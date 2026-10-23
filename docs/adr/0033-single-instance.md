# 33. Single instance of the desktop app

Date: 2026-10-23

## Status

Accepted

## Context

Starting Mailbeaver a second time (from the dock, the Start menu or a launcher while
the app sits in the tray) opened a second process whose servers failed on the ports
already taken by the first one, and showed a port error.

## Decision

- The desktop host registers `tauri-plugin-single-instance` (=2.5.3) as its first
  plugin. A second start hands over to the running app, which shows and focuses its main
  window; the second process quits before it starts any server.
- `restart_app` (ADR-0031) frees the plugin's lock with
  `tauri_plugin_single_instance::destroy` before `AppHandle::restart`, since a restart
  from the main thread skips `RunEvent::Exit`, where the plugin would free it.
- The headless server is not affected; it has no such plugin.

## Consequences

- No second instance, no port error on a second start.
- The lock is keyed by the app identifier; `pnpm tauri:dev` has its own (ADR-0034), so a
  development build and the installed app do not block each other.
