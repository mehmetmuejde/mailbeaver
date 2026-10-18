<p align="center">
  <img src="docs/assets/logo.png" alt="Mailbeaver logo" width="160">
</p>

<h1 align="center">Mailbeaver</h1>

<p align="center">
  Catches every mail your application sends during development — and shows it in a real
  mail client instead of delivering it.
</p>

<p align="center">
  <a href="https://github.com/mehmetmuejde/mailbeaver/releases/latest">Download</a> ·
  <a href="#install">Install</a> ·
  <a href="#send-mail-to-mailbeaver">Configure your app</a> ·
  <a href="LICENSE">MIT License</a>
</p>

---

Mailbeaver runs a local SMTP server. Point your application at it and every message lands
in Mailbeaver: nothing ever reaches a real inbox. It lives in your menu bar, shows the
newest mail right in the tray menu and opens a full mail client when you need it.

## Features

- **Mail client, not a log viewer** — Inbox, Archive and Trash; deleting moves to Trash,
  only Trash deletes for good. Optional automatic emptying after 1–365 days.
- **Projects** — sort mail by sender address, sender domain or SMTP username. Mail sent
  with SMTP credentials gets a project named after the username automatically.
- **Full-text search** across all folders over subject, sender, recipients and body,
  with filters such as `from:shop.test`, `subject:"order 12"`, `is:unread`,
  `has:attachment`, `project:shop` and `after:2026-10-01` — in the UI, the API and MCP.
- **Every view of a message** — sandboxed HTML preview with desktop and mobile width,
  plain text, all links with copy buttons (for confirmation and reset links), headers,
  raw source and attachments.
- **Tray menu** — unread count, the ten newest inbox messages with a blue dot for unread
  mail, grouped newest first, by sender or by project. A click opens the message.
- **Notifications** — a system notification for new inbox mail (all, or only selected
  projects; bursts share one notification) and the unread count on the app icon.
- **Link check** — see at a glance whether the links of a message answer.
- **API, Mailpit-compatible API and MCP server** — an [OpenAPI](https://www.openapis.org/) document at
  `/api/openapi.json` for tests, and an MCP server at `/mcp` so agents such as Claude Code
  can wait for, read and follow mail.
- **Logs** — Settings → Logs shows what the backend and the UI report: received and
  rejected mail, notifications, failed requests and errors.
- **Desktop extras** — pause capture (senders get a temporary error and retry), mark all
  read and empty Trash from the tray; download a message as `.eml`; rule suggestions
  from captured senders, domains and SMTP users; an opt-in update notice.
- **Live updates** — new mail appears instantly.
- **Local and private** — data stays in a SQLite database on your machine; the servers
  listen on `127.0.0.1` only.

## Install

### Homebrew (macOS)

```sh
brew install --cask mehmetmuejde/tap/mailbeaver
```

### Download

Get the installer for macOS (Apple Silicon or Intel), Windows or Linux from the
[latest release](https://github.com/mehmetmuejde/mailbeaver/releases/latest).

### Docker

The headless server has no tray, but the same mail client in the browser, the API and
the MCP server — for Docker Compose, CI or a shared dev server:

```sh
docker run -d --name mailbeaver \
  -p 127.0.0.1:1026:1026 -p 127.0.0.1:8026:8026 \
  -e MAILBEAVER_ALLOWED_HOSTS=localhost:8026 \
  -v mailbeaver-data:/data \
  ghcr.io/mehmetmuejde/mailbeaver:latest
```

Then open <http://localhost:8026>. A Compose setup next to your application is in
[`docs/docker/compose.yaml`](docs/docker/compose.yaml). The API has no login: publish
the ports on loopback or keep the container in a private network.

## Send mail to Mailbeaver

Configure your application's SMTP settings:

| Setting | Value |
|---|---|
| Host | `127.0.0.1` |
| Port | `1026` |
| Encryption | none |
| Username / password | optional — any values are accepted |

Mailbeaver never relays mail. The username, if you send one, can put the mail into a
project; the password is never stored.

<details>
<summary>Examples</summary>

```sh
# Laravel / PHP (.env)
MAIL_HOST=127.0.0.1
MAIL_PORT=1026

# Django (settings.py)
EMAIL_HOST = "127.0.0.1"
EMAIL_PORT = 1026

# Node.js (nodemailer)
nodemailer.createTransport({ host: "127.0.0.1", port: 1026, secure: false })
```

</details>

## Use it from tests and agents

- **OpenAPI:** `http://127.0.0.1:8026/api/openapi.json` describes every endpoint, e.g.
  `GET /api/messages?q=reset` to find a message in an end-to-end test.
- **Mailpit-compatible API:** test helpers written for [Mailpit](https://mailpit.axllent.org/)
  work when pointed at `http://127.0.0.1:8026` — `GET /api/v1/messages`,
  `/api/v1/search?query=`, `/api/v1/message/{id|latest}` (plus `/raw`, `/headers`,
  `/part/{id}`), `PUT` and `DELETE /api/v1/messages`, `DELETE /api/v1/search`,
  `/api/v1/info`. Deleting there is permanent, as in Mailpit.
- **Claude Code / MCP:**

  ```sh
  claude mcp add --transport http mailbeaver http://127.0.0.1:8026/mcp
  ```

  Tools: `list_messages`, `get_message`, `get_message_links`, `wait_for_message`,
  `mark_message_read`. None of them deletes anything.

## Configuration

Mailbeaver reads these environment variables at startup:

| Variable | Default | Meaning |
|---|---|---|
| `MAILBEAVER_SMTP_PORT` | `1026` | SMTP port |
| `MAILBEAVER_HTTP_PORT` | `8026` | Port of the HTTP API (and of the UI in the headless server) |
| `MAILBEAVER_BIND` | `127.0.0.1` | Address both servers listen on (`0.0.0.0` in the image) |
| `MAILBEAVER_MAX_MESSAGES` | `5000` | Messages kept; the oldest are dropped beyond this |
| `MAILBEAVER_MAX_MESSAGE_MB` | `25` | Largest accepted message |
| `MAILBEAVER_DATA_DIR` | `./data` | Database folder of the headless server (`/data` in the image) |
| `MAILBEAVER_ALLOWED_HOSTS` | – | Extra `Host` names the API accepts, e.g. `mail.test:8026` |
| `MAILBEAVER_PUBLIC_SMTP_ADDRESS` | – | SMTP address shown in the UI, e.g. behind port mapping |
| `MAILBEAVER_PUBLIC_HTTP_ADDRESS` | – | HTTP address shown in the UI |

The default ports differ from Mailpit's (1025/8025), so both can run side by side.

Ports and limits can also be set in the app (Settings → General → Ports and limits) and
apply after a restart; an environment variable for the same value wins. The update
check in the settings is off by default.

## Development

Requirements: Node.js (see `.nvmrc`), pnpm (version in `package.json`), Rust (pinned in
`rust-toolchain.toml`) and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
of your platform.

```sh
pnpm install
pnpm tauri dev                  # desktop app with hot reload
pnpm test                       # UI tests
cargo test --workspace          # Rust tests
pnpm build && cargo run -p mailbeaver-server   # headless server with the UI
```

The Rust code is a Cargo workspace: `crates/core` (SMTP, storage, API, MCP — no Tauri),
`src-tauri` (desktop app) and `crates/server` (headless server).

Architecture decisions are recorded in [`docs/adr/`](docs/adr/).

## Releases

A version tag builds and publishes everything:

```sh
pnpm release:version 1.2.3   # updates package.json, Cargo.toml and Cargo.lock
git commit -am "chore(release): 1.2.3"
git tag v1.2.3 && git push && git push --tags
```

The [release workflow](.github/workflows/release.yml) builds macOS (Apple Silicon and
Intel, signed and notarized), Windows and Linux, publishes the GitHub release and updates
the Homebrew cask in [`mehmetmuejde/homebrew-tap`](https://github.com/mehmetmuejde/homebrew-tap).

## Author

**Mehmet Müjde**
[muejde.mehmet@gmail.com](mailto:muejde.mehmet@gmail.com) ·
[github.com/mehmetmuejde](https://github.com/mehmetmuejde)

## License

[MIT](LICENSE) © 2026 Mehmet Müjde
