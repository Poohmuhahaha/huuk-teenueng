# Build context: repo root — needed so this Dockerfile can reach both app/ and
# ops/caddy/. From the repo root:
#   docker build -f app/Dockerfile .
# (docker compose in ops/ sets context: .. and dockerfile: app/Dockerfile)
# syntax=docker/dockerfile:1
# Node base image (real Node available) + Bun binary for install/build —
# same commands as CI (.github/workflows/ci.yml). Single lockfile: bun.lock.
FROM node:22-bookworm-slim AS build
WORKDIR /src
COPY --from=oven/bun:1-slim /usr/local/bin/bun /usr/local/bin/bun

# Leave VITE_API_URL empty to call the same origin (/api) through the web
# server; set it only when the API lives on a different host.
ARG VITE_API_URL=""
ARG VITE_USE_MOCK=""
ENV VITE_API_URL=${VITE_API_URL} \
    VITE_USE_MOCK=${VITE_USE_MOCK}

COPY app/package.json app/bun.lock ./
RUN bun install --frozen-lockfile
COPY app/ .
# Same as CI (.github/workflows/ci.yml): build via Bun.
RUN bun run build

FROM caddy:2-alpine
# Caddy config lives with the rest of the ops assets.
COPY ops/caddy/Caddyfile /etc/caddy/Caddyfile
COPY --from=build /src/dist /srv
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=5 \
    CMD wget -q -O /dev/null http://127.0.0.1:8080/ || exit 1
