# CLAUDE.md

## Project context

Mailbeaver catches every mail an application sends during development and shows it in a
mail client (Inbox, Archive, Trash, projects, search) instead of delivering it. It is a
Tauri 2 desktop app with a tray menu and a headless server for Docker; both host the
same Tauri-free core (SMTP server, SQLite storage, HTTP API, MCP). MIT licensed,
maintained by Mehmet Müjde.

## Stack

- Core and desktop host: Rust 1.99 (edition 2024), tokio, axum 0.8, rusqlite 0.40
  (bundled, FTS5), mail-parser, utoipa (OpenAPI), rmcp (MCP server), reqwest (link
  check), Tauri 2 — Cargo workspace: `crates/core`, `crates/server`, `src-tauri`
- UI: React 19 + TypeScript (strict), Vite, TanStack Query/Router/Form, Zod 4,
  shadcn/ui (Radix, Tailwind CSS v4, lucide-react), Biome, Steiger, knip, Vitest
- Package manager: pnpm (version pinned in `package.json`), Node per `.nvmrc`

## Commands

| Task | Command |
|---|---|
| Run the desktop app | `pnpm tauri:dev` (own identifier, ports 1027/8027) |
| Frontend checks | `pnpm check && pnpm typecheck && pnpm arch && pnpm knip` |
| Frontend tests | `pnpm test` / `pnpm test:coverage` |
| Rust checks | `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` |
| Headless server | `pnpm build && cargo run -p mailbeaver-server` |
| Docker image | `docker build -t mailbeaver .` |
| New version | `pnpm release:version 1.2.3`, commit, tag `v1.2.3`, push the tag |

## Conventions

- Architecture decisions live in `docs/adr/`; read them before changing a boundary.
- Rust: feature slices (`messages`, `projects`, `settings`, `links`) with domain, sqlite
  adapter, service and http modules; shared infrastructure in `db`, `http`, `changes`,
  `clock`, `stats`; frontends of the core in `smtp`, `http` and `mcp`. Every endpoint
  carries `#[utoipa::path]` so `/api/openapi.json` stays complete. All of this lives in
  `crates/core`; `src-tauri` is the only crate that uses Tauri, `crates/server` the
  headless host (ADR-0028).
- UI: Feature-Sliced Design (`app`, `pages`, `shared`). App code imports the design
  system from `@/shared/ui/<component>`, never `@/shared/ui/primitives` (shadcn output,
  regenerated with the shadcn CLI, never edited by hand).
- Server data only through TanStack Query; view state in typed URL search params.
- New dependencies are pinned exactly and at least one week old.
- TODOs: `TODO-MMUEJDE: <description>`.

## AI files in this repo

Tracked — decided by Mehmet on 2026-10-17 (ADR-0016). `CLAUDE.md`, `.claudeignore`,
`.claude/settings.json` and `.mcp.json` are committed; runtime data stays ignored.
Claude may appear as co-author in commits of this repository: `.claude/settings.json`
sets the trailer and `.ai-attribution-allowed` lets the commit guard pass. This
exception overrides the global rule against AI attribution for this repository only.
