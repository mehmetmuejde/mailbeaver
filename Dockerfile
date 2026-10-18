# syntax=docker/dockerfile:1
# Mailbeaver headless server (ADR-0028): SMTP on 1026; UI, API and MCP on 8026.
# Build: docker build -t mailbeaver .
# Base images are pinned by digest; update tag and digest together.

FROM node:24-bookworm-slim@sha256:a5f4366e834695d5f363b920c104953a535d3a9864d1bdf74566a0ebe01daee5 AS ui
WORKDIR /app
# pnpm in the version of `packageManager` in package.json.
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc ./
RUN pnpm install --frozen-lockfile
COPY index.html vite.config.ts tsconfig.json tsconfig.node.json components.json ./
COPY src ./src
RUN pnpm build

FROM rust:1.99-slim-bookworm@sha256:a24e292b4af7bdee351a31d3a6379972a5652f0ea22a1b6d6883324ac4f5097c AS server
WORKDIR /app
COPY rust-toolchain.toml Cargo.toml Cargo.lock clippy.toml ./
COPY crates ./crates
# Workspace member; only its manifest is read, nothing of Tauri is compiled.
COPY src-tauri ./src-tauri
COPY --from=ui /app/dist ./dist
RUN cargo build --release --locked -p mailbeaver-server \
    && mkdir -p /out/data

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
COPY --from=server /app/target/release/mailbeaver-server /usr/local/bin/mailbeaver-server
# Owned by the nonroot user (uid 65532), so a new volume is writable.
COPY --from=server --chown=65532:65532 /out/data /data
ENV MAILBEAVER_BIND=0.0.0.0 \
    MAILBEAVER_DATA_DIR=/data
# The volume keeps the database.
VOLUME ["/data"]
EXPOSE 1026 8026
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/mailbeaver-server", "healthcheck"]
ENTRYPOINT ["/usr/local/bin/mailbeaver-server"]
