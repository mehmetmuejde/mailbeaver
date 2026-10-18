# 29. Mailpit-compatible API

Date: 2026-10-17

## Status

Accepted

## Context

ADR-0003 chose an own REST API first and kept a Mailpit-compatible adapter for later.
Many projects already have test helpers for Mailpit (Laravel, Playwright, Cypress and
others) that call its `/api/v1`. Switching to Mailbeaver should not mean rewriting
them.

## Decision

- An adapter module `mailpit.rs` in the core serves the part of Mailpit's API that test
  helpers use, on the normal HTTP port and behind the same `Host` check and CORS rules:
  `GET /api/v1/messages`, `GET /api/v1/search?query=`, `GET /api/v1/message/{id}`
  (`latest` allowed) with `/raw`, `/headers` and `/part/{part}`, `PUT /api/v1/messages`
  (read state), `DELETE /api/v1/messages` (ids or everything), `DELETE /api/v1/search`
  and `GET /api/v1/info`.
- Field names and shapes follow Mailpit (`ID`, `From.Address`, `messages_count`, …).
  Ids are Mailbeaver's numeric ids as strings; projects appear as tags.
- Mailpit has no folders, so the adapter sees every message outside Trash.
- As in Mailpit, deleting is permanent (the adapter moves to Trash and deletes from
  there) and reading a message marks it read.
- The search uses Mailbeaver's syntax (ADR-0027), whose common filters match Mailpit's.
- The endpoints are part of the OpenAPI document under the tag `mailpit`.

## Consequences

- Tools only need the base URL changed to port 8026.
- Not covered: Mailpit's tags API, release/relay, chaos testing, HTML/link/spam checks,
  inline parts and `Cc`/`Bcc` in list items (the detail has `Cc`). They can follow
  when someone needs them.
- A test deleting through `/api/v1` also removes the message from the UI for good.
