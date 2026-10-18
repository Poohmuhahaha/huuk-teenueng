# Huuk — deterministic command runner (Master Engineering SOP §1.3).
set dotenv-load := false

# Start local PostgreSQL container
dev-db:
    docker compose -f deploy/docker-compose.dev.yml up -d postgres

# Run database migrations
migrate:
    cd server && sqlx migrate run

# Generate TypeScript bindings from Rust structs via ts-rs
gen-types:
    cd server && cargo test --lib export_bindings

# Verify TypeScript bindings match Rust structs (CI Gate)
check-types:
    just gen-types
    git diff --exit-code app/src/api/types/

# Run backend and frontend concurrently
dev:
    bash ops/scripts/dev.sh

# Full local CI gate (must pass before any commit or PR)
ci:
    cd server && cargo fmt --all -- --check
    cd server && cargo clippy --all-targets -- -D warnings
    just check-types
    cd server && cargo test
    cd app && bun install --frozen-lockfile
    cd app && bun run typecheck
    cd app && bun run test
    cd app && bun run build

# Alias for backward compatibility
verify: ci

# Rust tests only
test-server:
    cargo test --manifest-path server/Cargo.toml

# Frontend tests only
test-app:
    cd app && bun run test

# Production builds (release server binary + built SPA)
build:
    cargo build --release --manifest-path server/Cargo.toml
    cd app && bun run build
