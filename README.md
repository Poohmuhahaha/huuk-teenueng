<div align="center">

# Huuk

**Open-Source Content Planning, Campaign Management & Multi-Platform Social Studio**

[![Status: WIP](https://img.shields.io/badge/Status-Active%20Development%20(WIP)-f59e0b)](https://github.com/Poohmuhahaha/huuk-teenueng)
[![CI/CD Pipeline](https://github.com/Poohmuhahaha/huuk-teenueng/actions/workflows/ci-cd.yml/badge.svg)](https://github.com/Poohmuhahaha/huuk-teenueng/actions/workflows/ci-cd.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-emerald.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Backend-Rust%20%7C%20Axum%200.8%20%7C%20SQLx%200.8-orange)](https://github.com/tokio-rs/axum)
[![Vue 3](https://img.shields.io/badge/Frontend-Vue%203.5%20%7C%20TanStack%20Query%20v5-42b883)](https://vuejs.org)
[![PostgreSQL](https://img.shields.io/badge/Database-PostgreSQL%2016-336791)](https://www.postgresql.org)
[![Caddy](https://img.shields.io/badge/Ingress-Caddy%202-1f883d)](https://caddyserver.com)

<p align="center">
  <img src="https://i.pinimg.com/1200x/5e/1c/4d/5e1c4d3cd83e7f0f439b41089c1cbe5c.jpg" alt="Huuk Banner" width="100%" />
</p>

</div>

> [!WARNING]
> **Project Status: Active Development (Work in Progress / Pre-Alpha)**
> Huuk is currently under heavy active development and is **not yet feature-complete**. Core architecture, database schemas, API contracts, and UI workflows may change frequently before the first stable release (`v1.0.0`).

---

## Overview

**Huuk** is a local-first, self-hostable content operations and social publishing workspace built for creators, agencies, and marketing teams. It unifies editorial calendars, multi-platform post composition, campaign tracking, brand asset kits, budget/finance ledgers, and performance analytics into a single high-speed web application.

### Key Features

- **Multi-Platform Content Calendar & Feed** — Plan, schedule, preview, and publish content across TikTok, Instagram, Facebook, YouTube, Shopee, Line VOOM, and Lemon8 with drag-and-drop calendar and feed views.
- **Visual Studio & Brand Kit** — Design slides, manage custom brand palettes, upload custom typography (`.ttf`, `.otf`, `.woff2`), and maintain reusable brand assets per workspace.
- **Campaign &Ads Management** — Organize deliverables by campaign, track ad spend vs. ROAS, and monitor milestone progress across channels.
- **Idea Backlog & Content Pipeline** — Capture hooks, scripts, and inspirations in a structured backlog and promote them directly into scheduled posts.
- **Workspace Finance & RBAC** — Track production income/expenses per campaign and collaborate safely with role-based access control (`Owner`, `Admin`, `Editor`, `Viewer`).
- **Automated Publish Scheduler** — Background Tokio scheduler with retry backoffs, OAuth token refresh, and webhook signature verification (`HMAC-SHA256`).

---

## System Architecture

Huuk is engineered around a strict **Same-Origin Reverse Proxy Topology** and **Contract-First Type Pipeline**:

```mermaid
flowchart LR
    Browser["Browser (Vue 3 SPA)"]
    Caddy["Caddy 2 Edge Proxy\nTLS + Zstd/Gzip + Security Headers"]
    Static["Static Assets (/assets/*)\nImmutable 1-Year Cache"]
    Axum["Rust API (Axum 0.8 + MiMalloc)\nHttpOnly Cookie Auth + Rate Limiting"]
    PG[("PostgreSQL 16\nSQLx Pool + Session Guardrails")]

    Browser -->|"HTTPS (Same-Origin)"| Caddy
    Caddy -->|"/assets/* & /index.html"| Static
    Caddy -->|"/api/* & /healthz"| Axum
    Axum -->|"SQLx (5s statement / 3s lock timeout)"| PG
```

### Engineering Highlights

| Layer | Technology & Standards |
| :--- | :--- |
| **Edge / Reverse Proxy** | **Caddy 2** (`deploy/Caddyfile`) — TLS termination, `zstd`/`gzip` compression, `HSTS`/`CSP`/`X-Frame-Options` headers, `immutable` cache on `/assets/*`, `no-store` on `/index.html`, unbuffered SSE on `/api/agent/stream/*`, and active `/healthz` checks. |
| **Frontend SPA** | **Vue 3.5** (`<script setup lang="ts">`) + **TanStack Query v5** + **Tailwind CSS v4** semantic design tokens + **Vite** manual vendor chunk splitting (`app/`). Every data view enforces explicit **4-State UI** (`isPending`, `isError`, `isEmpty`, `data`). |
| **Type-Safe Contract** | **ts-rs** exports Rust DTOs (`#[derive(TS)]`) directly to `app/src/api/types/`. **Zod** schemas in `app/src/api/schemas/` enforce compile-time equality (`AssertEqual<ZodType, TsRsType>`) and runtime response validation. |
| **Authentication** | **HttpOnly + Secure + SameSite=Lax** session cookies (`axum-extra`) paired with a client-side **single-flight `401` refresh queue** (`app/src/api/client.ts`). Zero tokens are ever stored in `localStorage` or `sessionStorage`. |
| **Backend API** | **Rust (Axum 0.8 + Tokio)** with `mimalloc` global allocator, zero production `.unwrap()`/`.expect()`, and `tokio::task::spawn_blocking` isolation for CPU/filesystem work (`server/`). |
| **Database & Migrations** | **PostgreSQL 16** + **SQLx 0.8** (`server/migrations/`). Zero-downtime **Expand-and-Contract** migrations run via a dedicated one-off container (`--run-migrations-only`) prior to API startup. |

---

## Repository Structure

```text
huuk/
├─ .github/workflows/ci-cd.yml   # 4-stage CI/CD pipeline (Lint/Types -> Postgres Tests -> Build -> Deploy)
├─ deploy/
│  ├─ Caddyfile                  # Production Caddy 2 reverse proxy configuration
│  ├─ docker-compose.dev.yml     # Local PostgreSQL 16 development database
│  └─ docker-compose.prod.yml    # Production stack (Postgres 16, one-off Migration, Axum API, Caddy)
├─ server/                       # Rust (Axum 0.8 + SQLx 0.8 + MiMalloc) backend
│  ├─ Cargo.toml                 # Crate manifest with Thin-LTO release optimizations
│  ├─ Dockerfile                 # Multi-stage cargo-chef + musl + Alpine 3.20 (non-root appuser)
│  ├─ migrations/                # SQLx PostgreSQL schema migrations
│  └─ src/
│     ├─ main.rs                 # Entrypoint, --run-migrations-only mode, graceful SIGTERM shutdown
│     ├─ db.rs                   # PgPoolOptions guardrails & PostgreSQL state persistence
│     ├─ error.rs                # Unified ApiErrorResponse contract
│     ├─ model.rs                # Domain models & ts-rs DTOs (#[derive(TS)])
│     ├─ handlers.rs             # REST endpoints, HttpOnly cookie auth, multipart uploads
│     ├─ oauth.rs                # Social platform OAuth & publishing integrations
│     ├─ scheduler.rs            # Background post publishing worker
│     └─ store.rs                # Core workspace, campaign, calendar, and RBAC engine
├─ app/                          # Vue 3 + TanStack Query + Tailwind CSS v4 + Zod SPA
│  ├─ tailwind.config.ts         # Semantic design tokens
│  ├─ vite.config.ts             # Vite build & vendor chunk splitting configuration
│  └─ src/
│     ├─ api/
│     │  ├─ client.ts            # customFetch with single-flight 401 token refresh queue
│     │  ├─ types/               # Auto-generated TypeScript bindings from Rust (ts-rs)
│     │  └─ schemas/             # Zod schemas + AssertEqual<Zod, TsRs> compile-time gates
│     ├─ components/             # Reusable UI primitives & studio modals
│     ├─ composables/            # TanStack Query hooks & reactive workspace state
│     ├─ pages/                  # Route views (Dashboard, Calendar, Studio, Campaigns, Finance, etc.)
│     └─ tests/                  # Vitest unit & component test suite
├─ ops/                          # Operational scripts (dev runner, backup/restore, smoke checks)
├─ CONTRIBUTING.md               # Contribution workflow & coding standards
├─ SECURITY.md                   # Vulnerability disclosure policy
├─ LICENSE                       # MIT License
└─ justfile                      # Deterministic command runner
```

---

## Getting Started

### Prerequisites

- **Rust** `1.85+` (`rustup`)
- **Bun** `1.2+`
- **Docker & Docker Compose** (for PostgreSQL 16 and containerized deployment)
- **just** (optional, for running `justfile` recipes)

### 1. Local Development

Start the development stack (Rust API on `:8787` + Vite HMR frontend on `:5173`):

```bash
# Install frontend dependencies
cd app && bun install && cd ..

# Optional: start local PostgreSQL 16 and apply SQLx migrations
docker compose -f deploy/docker-compose.dev.yml up -d postgres
DATABASE_URL=postgres://huuk:huuk_dev_password@localhost:5432/huuk \
  cargo run --manifest-path server/Cargo.toml -- --run-migrations-only

# Start backend + frontend concurrently
bun run dev
```

Open **`http://localhost:5173`** in your browser.

### 2. Production Deployment (Docker Compose)

Run the full production topology (**PostgreSQL 16** + **One-Off Migration Container** + **Rust API** + **Caddy 2 Reverse Proxy**):

```bash
# 1. Execute database migrations via one-off container
docker compose -f deploy/docker-compose.prod.yml run --rm migration

# 2. Launch the production stack
docker compose -f deploy/docker-compose.prod.yml up -d --build
```

---

## Developer Commands (`justfile`)

All verification, code generation, and build workflows are standardized in [`justfile`](justfile):

| Command | Description |
| :--- | :--- |
| `just dev` | Start the Rust backend (`:8787`) and Vue 3 Vite dev server (`:5173`) |
| `just dev-db` | Start the local PostgreSQL 16 container via `deploy/docker-compose.dev.yml` |
| `just gen-types` | Export TypeScript `.d.ts` bindings from Rust structs via `ts-rs` |
| `just check-types` | Regenerate `ts-rs` bindings and fail if `app/src/api/types/` has uncommitted drift |
| `just test-server` | Run the Rust unit and integration test suite (`cargo test`) |
| `just test-app` | Run the frontend Vitest suite (`bun run test`) |
| `just verify` / `just ci` | Run the full CI gate (`cargo fmt`, `clippy -D warnings`, `check-types`, `cargo test`, `vue-tsc`, `vitest`, `vite build`) |

---

## Contributing & Security

- **Contributing**: Please review [CONTRIBUTING.md](CONTRIBUTING.md) and ensure `just verify` passes before opening a Pull Request.
- **Security**: To report a security vulnerability, please follow the responsible disclosure instructions in [SECURITY.md](SECURITY.md).

## License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for full details.
