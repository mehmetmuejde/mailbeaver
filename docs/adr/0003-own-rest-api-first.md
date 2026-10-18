# 3. Own REST API first, Mailpit-compatible API later

Date: 2026-10-17

## Status

Accepted

## Context

Mailpit is the tool most teams know, and some test suites call its API. Copying its API
shape now would tie our UI and data model to another product's decisions.

## Decision

The core exposes its own REST API under `/api/…`, shaped by what our UI needs. A
Mailpit-compatible API may be added later as a separate adapter that translates to the
same mailbox calls.

## Consequences

- Our API can follow our data model (folders, mailboxes, search) without compromises.
- Test suites that call the Mailpit API need the compatibility adapter first.
