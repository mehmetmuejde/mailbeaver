# 30. Logs in the database, shown in the settings

Date: 2026-10-17

## Status

Accepted

## Context

When mail or a notification does not arrive, the reason was only visible on the
console of `pnpm tauri dev`, and not at all in the installed app. Errors of the UI
(failed requests, exceptions) were not recorded anywhere.

## Decision

- A table `logs` (time, level, source `backend`/`frontend`, target, message) keeps the
  newest 10,000 entries; older ones are deleted on each write. Levels: `error`,
  `warn`, `info`.
- Backend: a `tracing` layer in the core (`logs::database_layer`) copies events into
  the table. Each host adds it to its subscriber next to the console output. Events go
  through a bounded channel and are written in batches, so logging never waits for
  SQLite; when the channel is full, entries are dropped. Stored: Mailbeaver's own
  events from `info`, other crates' from `warn`. The writer's own events are skipped.
- UI: `POST /api/logs` stores an entry as `frontend`, limited to 60 per minute.
  Reported: failed queries and mutations (4xx as warnings), uncaught errors, unhandled
  promise rejections and render errors. A report that fails is dropped, never
  reported again.
- `GET /api/logs` (filter by minimum level and source) and `DELETE /api/logs`;
  Settings → Logs lists, filters, copies and clears them and refreshes every three
  seconds while open.
- Logged are envelope data (sender, recipients, SMTP user), decisions (e.g. why no
  notification was shown) and errors — never mail content or passwords.
- "Reset Mailbeaver" (ADR-0018) clears the log too.

## Consequences

- No new dependency: `tracing-subscriber` was already used by the hosts.
- Debug output stays on the console only.
- The schema was consolidated into one `0001_initial.sql` before the first release;
  databases of development builds with schema version 2 or 3 must be removed once.
