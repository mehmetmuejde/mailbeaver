# 16. Claude Code files are part of the repository

Date: 2026-10-17

## Status

Accepted

## Context

The project is developed with Claude Code. By default the maintainer keeps AI helper
files out of every repository and blocks AI attribution in commits.

## Decision

For this repository `CLAUDE.md`, `.claudeignore`, `.claude/settings.json` and
`.mcp.json` are committed (negations in `.gitignore`); runtime data under `.claude/` stays
ignored. Claude may appear as co-author: `.claude/settings.json` sets the
`Co-Authored-By` trailer and `.ai-attribution-allowed` lets the maintainer's commit guard
pass for this repository only.

## Consequences

- Contributors using Claude Code get the same project instructions and the shadcn MCP
  server.
- Claude shows up as a contributor on GitHub.
