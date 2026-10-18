# 10. Mailboxes per project via the SMTP AUTH username

Date: 2026-10-17

## Status

Superseded by ADR-0012

## Context

Developers often run several projects against one Mailcatcher and want to tell their
mail apart. The SMTP client of each project can be configured with credentials.

## Decision

- The username of a successful SMTP AUTH (PLAIN or LOGIN) is the project key and is
  stored in `messages.mailbox`. Mail sent without AUTH has no mailbox (catch-all).
- Passwords are still accepted unchecked and never stored.
- The username is decoded with the `base64` crate; it is trimmed and must be 1–64
  characters without control characters, otherwise the message lands in the catch-all.
- The inbox can be shown as one list, grouped by sender or grouped by project, and
  filtered by project. `GET /api/mailboxes` lists known projects with counts.

## Consequences

- Projects need SMTP credentials configured to get their own mailbox.
- One new dependency (`base64`).
