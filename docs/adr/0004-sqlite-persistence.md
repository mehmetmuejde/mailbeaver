# 4. SQLite persistence with a retention limit

Date: 2026-10-17

## Status

Accepted

## Context

Captured mail should survive a restart of the app. The data is local, single-user and
small; a database server would be out of proportion.

## Decision

Messages are stored in SQLite via `rusqlite` with the `bundled` feature (no system
SQLite), in WAL mode, in the app data directory. Persistence is always on. The schema
version lives in `PRAGMA user_version`; a database with a newer version is refused.
A retention limit (`MAILCATCHER_MAX_MESSAGES`, default 5000) drops the oldest messages.

## Consequences

- No external service is needed; the database is one file.
- Schema changes need a migration step for existing databases.
- All store calls are synchronous and run on the blocking thread pool.
