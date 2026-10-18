# 13. Rust core organized in feature slices

Date: 2026-10-17

## Status

Accepted

## Context

The first version of the Rust code was split by technical layer (`store`, `mailbox`,
`api`). Every feature touched all of them, and the coming projects feature would have
made that worse. The Rust guidelines of this project prefer feature slices; the layered
command core with workspace crates is meant for apps whose frontends share a domain in
process, while Mailtomb's UI talks HTTP (ADR-0002).

## Decision

One crate (`src-tauri`) with these modules:

- Features: `messages`, `projects`, `settings` — each with `domain` (pure types and
  rules), `sqlite` (adapter), `service` (use cases) and `http` (axum handlers, DTOs).
  Other modules use a feature only through its facade (`messages.rs`, …).
- Shared infrastructure: `db` (connection, migrations, `DbError`), `http` (router
  assembly, `ApiError`, host check, CORS, SSE), `changes` (change feed), `clock`,
  `config`, `smtp` (protocol adapter) and `server` (starts everything).
- Host: `desktop` (Tauri, tray) is the only module that uses Tauri.

No storage traits: SQLite is the only backend and tests run against an in-memory
database. The split into workspace crates follows when the headless binary arrives: it
must build without Tauri.

## Consequences

- A feature change stays inside its folder; the compiler enforces the facades.
- The tray menu layout is plain Rust and unit-tested; only the menu construction needs
  Tauri.
