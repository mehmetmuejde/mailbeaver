# 9. Full-text search with SQLite FTS5

Date: 2026-10-17

## Status

Accepted

## Context

Users search for a mail by subject, sender, recipient or a word in the body. Parsing
every stored message per search does not scale to thousands of messages.

## Decision

- At receive time the plain-text body is extracted once into `messages.body_text`
  (text part, otherwise the HTML part converted to text).
- An external-content FTS5 table `messages_fts(subject, header_from, header_to,
  body_text)` indexes these columns and is kept in sync by triggers.
- The search query is split into words; every word becomes a quoted prefix term, so
  user input can never form FTS5 syntax. All words must match.

## Consequences

- Searching is fast and independent of message size.
- The database grows by the extracted text and the index.
- The migration to schema version 2 extracts the text of existing messages once.
