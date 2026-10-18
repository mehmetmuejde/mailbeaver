# 18. Reset all data instead of deleting the database file

Date: 2026-10-17

## Status

Accepted

## Context

Users want to start over with a clean Mailtomb: no messages, no projects, default
settings. Deleting the database file while the servers run would break their open
connection and leave the WAL and shared-memory files behind.

## Decision

Settings → Danger zone offers "Delete all messages" and "Reset Mailtomb". The reset
(`DELETE /api/data`) removes all messages, projects, rules and settings in one
transaction, restarts the message ids, rebuilds the search index and runs `VACUUM` so the
file shrinks. The schema stays. The endpoint is a `DELETE` so a foreign web page cannot
trigger it with a plain form post. The theme stays, as it is stored on the device.

## Consequences

- A reset is irreversible; the UI asks for confirmation.
- Every client reloads all data after the `reset` change event.
