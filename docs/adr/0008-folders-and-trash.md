# 8. Folders, trash and permanent deletion

Date: 2026-10-17

## Status

Accepted

## Context

The UI behaves like a mail program. Deleting a message by accident should be easy to
undo, while really removing data stays a deliberate step.

## Decision

- Every message is in exactly one folder: `inbox`, `archive` or `trash`
  (`messages.folder` with a `CHECK` constraint, default `inbox`).
- Deleting a message moves it to Trash without confirmation; `trashed_at` records when.
- Messages are deleted permanently only from Trash: a single message without
  confirmation, "Empty trash" with a confirmation dialog.
  `DELETE /api/messages/{id}` answers `409 Conflict` for messages outside Trash.
- "Delete all messages" exists only in Settings → Danger zone, behind a confirmation.
- Trash can be emptied automatically after a configurable number of days (setting
  `trashRetentionDays`, off by default, 1–365). A background task checks hourly.
- The tray shows the unread count of the inbox only.

## Consequences

- Schema version 2 adds the columns; existing messages land in the inbox.
- The retention limit (ADR-0004) still counts messages of all folders.
