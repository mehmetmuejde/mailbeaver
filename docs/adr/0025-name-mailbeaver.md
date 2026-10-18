# 25. The project is called Mailbeaver

Date: 2026-10-17

## Status

Accepted

## Context

ADR-0011 named the project Mailtomb. Together with the first artwork — a beaver
carrying an envelope — the maintainer chose a new name. The name must
again be free on GitHub (no repository contains it), npm, crates.io and Homebrew; it is.
No build under the name Mailtomb was released.

## Decision

The project is called **Mailbeaver**. The beaver stores every mail and never passes it
on. Derived identifiers, replacing those of ADR-0011:

- Repository `mehmetmuejde/mailbeaver`, Homebrew cask `mailbeaver` in
  `mehmetmuejde/homebrew-tap`
- App identifier `io.github.mehmetmuejde.mailbeaver`
- Rust package `mailbeaver` (library `mailbeaver_lib`), npm package `mailbeaver`
- Environment variables `MAILBEAVER_*`, database file `mailbeaver.db`, MCP server name
  `mailbeaver`

Artwork: the mascot (`src/shared/ui/logo/mailbeaver.png`, app icons generated with
`pnpm tauri icon` into `src-tauri/icons`), a template tray icon
(`src-tauri/icons/tray-icon.png`) and a text wordmark — "Mail" in slate (white in dark
mode), "beaver" in blue — set in the system's rounded font, so the UI loads no web font
and keeps working offline under its CSP.

## Consequences

- Earlier ADRs still say Mailtomb; read it as Mailbeaver, with the identifiers above.
- The new app identifier means a new data directory: data of Mailtomb builds is not
  taken over.
- On systems without a rounded font the wordmark falls back to the system font.
