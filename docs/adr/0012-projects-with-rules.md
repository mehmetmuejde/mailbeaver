# 12. Projects with assignment rules

Date: 2026-10-17

## Status

Accepted. Supersedes ADR-0010.

## Context

ADR-0010 tied a project to the SMTP AUTH username. Many applications cannot send
credentials, and users want to sort mail by sender as well, e.g. all mail from
`shop.example` into one project.

## Decision

- A **project** has a unique name (case-insensitive, 1–64 characters) and up to 50
  **rules**. Rule kinds: exact **sender address**, **sender domain** (also matches its
  subdomains) and **SMTP user**. Rule values are stored in lowercase.
- A rule value belongs to **one project only** (`UNIQUE (kind, value)`), so a sender
  never lands in two projects.
- Assignment order: SMTP user before exact address before domain; among domain rules
  the most specific wins. The sender is the address of the first `From` header, or the
  envelope sender when there is none.
- Mail from an SMTP user no rule knows creates (or joins) a project named after the
  user, with an SMTP-user rule — the zero-setup behaviour of ADR-0010 stays.
- Every change to projects or rules re-assigns all stored messages in the same
  transaction; deleting a project leaves its messages without one.
- Projects are managed under Settings; API: `GET/POST /api/projects`,
  `PUT/DELETE /api/projects/{id}`; lists filter with `?project=<id>|none`.

## Consequences

- Rules apply to old and new mail alike; there is no manual per-message assignment.
- Re-assignment reads every message once per change, which is fine for the retention
  limit of a few thousand messages.
