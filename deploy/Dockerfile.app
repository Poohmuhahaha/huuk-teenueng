# syntax=docker/dockerfile:1
# All-in-one Huuk image: Rust API + built Vue SPA.
#
# Build on the target architecture (AWS t4g = linux/arm64, t3 = amd64) so no
# cross-compilation is involved:
#   docker build -f deploy/Dockerfile.app -t huuk:local .
#
# The server serves the SPA itself via STATIC_DIR, so no nginx/Caddy is needed.
# Runtime data (store.json + images/ + fonts/) lives on the /data volume.

# ---- frontend ----
# Use real Node here: vue-tsc's language plugin does not work with Bun's
# `node` shim (the Bun slim image symlinks node -> bun).
FROM node:22-bookworm-slim AS web
WORKDIR /src
COPY --from=oven/bun:1-slim /usr/local/bin/bun /usr/local/bin/bun
COPY app/package.json app/bun.lock ./
RUN bun install --frozen-lockfile
COPY app/ ./
RUN bun run build

# ---- backend ----
FROM rust:1-bookworm AS server
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential pkg-config cmake perl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Cache crate dependencies in their own layer. A dummy main keeps the
# manifest/target graph identical while dependencies compile.
COPY server/Cargo.toml server/Cargo.lock ./
RUN mkdir -p src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release --locked --bin content-planner-server || true

COPY server/src ./src
COPY server/migrations ./migrations
COPY server/openapi.yaml ./
COPY server/swagger ./swagger
RUN cargo build --release --locked --bin content-planner-server

# ---- runtime ----
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl passwd tzdata \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 10001 app \
    && useradd --system --uid 10001 --gid app --home-dir /app --shell /usr/sbin/nologin app

WORKDIR /app
COPY --from=server /app/target/release/content-planner-server /app/content-planner-server
COPY --from=web /src/dist /app/dist
RUN mkdir -p /data && chown -R app:app /app /data

USER app
ENV BIND_ADDR=0.0.0.0 \
    PORT=8787 \
    STATIC_DIR=/app/dist \
    DATA_FILE=/data/store.json

VOLUME ["/data"]
EXPOSE 8787

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8787/api/health >/dev/null || exit 1

ENTRYPOINT ["/app/content-planner-server"]
