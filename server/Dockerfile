# Master Engineering SOP §5.2 — Multi-Stage Rust Container Build (cargo-chef + musl + alpine + appuser)
FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev
ENV CARGO_NET_RETRY=10 \
    CARGO_HTTP_MULTIPLEXING=false

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build & cache dependencies layer
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Build application layer
COPY Cargo.toml Cargo.lock openapi.yaml ./
COPY migrations ./migrations
COPY swagger ./swagger
COPY src ./src
ENV SQLX_OFFLINE=true
RUN cargo build --release --target x86_64-unknown-linux-musl --locked --bin content-planner-server

# Minimal runtime stage
FROM alpine:3.20 AS runtime
RUN apk add --no-cache ca-certificates tzdata
WORKDIR /app

RUN addgroup -S appgroup && adduser -S appuser -G appgroup && mkdir -p /app/data && chown -R appuser:appgroup /app
USER appuser

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/content-planner-server /app/server
ENV BIND_ADDR=0.0.0.0 \
    PORT=8787 \
    DATA_FILE=/app/data/content-planner.json
EXPOSE 8787
CMD ["/app/server"]
