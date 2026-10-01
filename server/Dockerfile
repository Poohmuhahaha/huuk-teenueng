# Build context: server/ (from repo root: docker build -f server/Dockerfile server)
# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
# Embedded docs assets referenced with include_str! from src/.
COPY openapi.yaml ./
COPY swagger ./swagger
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /app app
WORKDIR /app
COPY --from=build /src/target/release/content-planner-server /usr/local/bin/content-planner-server
RUN mkdir -p /app/data && chown -R app:app /app
USER app
ENV BIND_ADDR=0.0.0.0 \
    PORT=8787 \
    DATA_FILE=/app/data/content-planner.json
EXPOSE 8787
VOLUME ["/app/data"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8787/api/health || exit 1
CMD ["content-planner-server"]
