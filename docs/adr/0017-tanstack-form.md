# 17. TanStack Form for forms

Date: 2026-10-17

## Status

Accepted

## Context

Managing projects and their rules is the first form of the UI. The frontend guidelines
allow React Hook Form or TanStack Form, one per project.

## Decision

Forms use TanStack Form with a Zod schema per form as form-level validator. The server
validates again; its error message is shown at the form.

## Consequences

- Same library family as TanStack Query and Router.
- One more runtime dependency (`@tanstack/react-form`).
