# 11. The project is called Mailtomb

Date: 2026-10-17

## Status

Superseded by ADR-0025

## Context

The working name "Mailcatcher" belongs to a well-known Ruby tool (sj26/mailcatcher) that
also ships as a Homebrew formula; publishing under it would collide on GitHub and in
Homebrew. "Mailpit" is the tool this project is compared with. The name must be free on
GitHub (no repository contains it), npm, crates.io and Homebrew.

## Decision

The project is called **Mailtomb**: mail goes in and never reaches a real recipient.
Derived identifiers:

- Repository `mehmetmuejde/mailtomb`, Homebrew cask `mailtomb` in `mehmetmuejde/homebrew-tap`
- App identifier `io.github.mehmetmuejde.mailtomb`
- Rust package `mailtomb` (library `mailtomb_lib`), npm package `mailtomb`
- Environment variables `MAILTOMB_*` (replacing `MAILCATCHER_*` from ADR-0004 and
  ADR-0005), database file `mailtomb.db`

## Consequences

- The new app identifier means a new data directory; databases of the earlier POC builds
  are not taken over.
- The schema starts again at version 1 (`db/migrations/0001_initial.sql`), since no
  build with the earlier schema was released.
