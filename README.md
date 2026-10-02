# Huuk by teenueng

Content planner — Vue 3 frontend + Rust backend.

| Part | Folder | What it is |
|---|---|---|
| dev | `app/` | Canonical frontend — Vue 3 + TanStack Query, client-ready design system (see `app/README.md`) |
| dev | `server/` | Rust (Axum) REST backend (see `server/README.md`) |
| desktop | `desktop/` | Tauri desktop shell (see `desktop/`) |
| dev | `ops/` | Operations — `deploy/` (compose env, Caddy, systemd), `nginx/`, `scripts/` (dev, deploy, install, backup, restore, smoke), `docker-compose.yml` (see `ops/README.md`) |
| data | `data/` | Runtime backend snapshot (`content-planner.json`, git-ignored — the live store, not source) |
| backup | `backups/` | Timestamped snapshots (git-ignored, mode 600 — see `docs/DEPLOYMENT.md` § backups) |
| other | `docs/` | Developer + operations docs, including `docs/DEPLOYMENT.md` |

Status: all 14 user stories (US-001…US-014) are implemented — 94 Rust tests + 188 frontend tests green,
real email+password accounts (Argon2id, session tokens, rate-limited login, account revocation,
self-service profile), configurable auth/permissions with an admin recovery token, full-screen
social platform login pages, and a credential-optional OAuth 2.0 flow (Meta/Facebook/Instagram,
Google/YouTube, TikTok wired; add credentials to go live).

**Built-in CMS with a client portal:** articles/pages/notes with a Markdown editor + live preview,
autosave, revision history/restore, scheduled publishing, SEO fields and a public read-only
delivery page (`/#/read/{slug}`). Clients get their own **Content Studio** (`/#/studio`) and a
dedicated **Client** role; admins onboard them from Settings by creating a sign-in account
(a temporary password is generated and shown once).

Production hardening is in place: durable atomic JSON snapshots (`DATA_FILE`), env-driven
configuration, restrictive-by-default CORS/security headers/body limits/request timeouts,
graceful shutdown, health + readiness probes, structured audit logging, server-side validation,
and collision-free ids. `ops/scripts/dev.sh` runs the explicit demo mode; deployments run in production mode
(`DEMO_MODE` unset → auth required, demo accounts removed, registration closed, first Owner
bootstrapped from `ADMIN_EMAIL`/`ADMIN_PASSWORD`). See `server/README.md` § Going to production.

## Run everything — one command

```sh
bun run dev          # or: npm run dev / bash ops/scripts/dev.sh
```

That builds/starts the Rust backend on `:8787` in demo mode, waits for its health check, then
starts the Vue dev server (using **bun** when available, npm otherwise) with `VITE_API_URL` wired
to it. **Ctrl+C stops both** and the backend flushes its snapshot before exiting.

Production — no Docker needed (the binary serves the API **and** the frontend):

```sh
sudo ops/scripts/install.sh     # build + install + systemd service, generated admin secrets
# → http://localhost:8787, config in /etc/content-planner.env
ops/scripts/install.sh --help   # port, prefix, public host, updates, …
```

Production — Docker Compose (API + web + optional Caddy TLS):

```sh
ops/scripts/deploy.sh init      # create ops/deploy/.env.production with generated admin secrets
ops/scripts/deploy.sh up        # http://localhost:8080
ops/scripts/deploy.sh up --tls  # Caddy on 80/443 with automatic HTTPS
ops/scripts/deploy.sh smoke     # verify the running deployment
```

Full guide (all paths incl. Podman, env reference, client onboarding, backups/restore, updates,
staging rehearsal, troubleshooting): **`docs/DEPLOYMENT.md`**.

```sh
bun run dev:mock     # frontend only, in-memory mock data (no backend)
bun run dev:server   # backend only
bun run app          # frontend only (mock), e.g. for UI work
bun run server       # backend only via cargo
bun run test         # Rust tests (94) + frontend tests (188)
bun run build        # release backend + production frontend build
```

First-time frontend install (bun or npm — either works):

```sh
cd app && bun install       # or: npm install
```

Ports: backend `8787` (override with `PORT=… npm run dev`), frontend default Vite port
(5173, auto-increments if busy).

## Requirements

- **Bun** (preferred) or Node 20+ with npm
- Rust 1.75+ with cargo (only for `server/` or the full `run dev`; `run dev:mock` needs no Rust)

## How the two sides connect

`app/src/api/index.ts` picks the data source: unset `VITE_API_URL` → in-memory mock
(`app/src/mock/api.ts`); set → HTTP client (`app/src/api/http.ts`) → the Rust API. Both implement
the same functions, so pages never change. `ops/scripts/dev.sh` sets the var for you.

```sh
# manual alternative
cd server && cargo run
cd app && echo 'VITE_API_URL=http://localhost:8787' > .env.local && bun run dev  # or npm run dev
```

## More documentation

- `docs/DEPLOYMENT.md` — production deployment (Docker Compose, TLS, backups, smoke tests)
- `docs/00_STATUS.md` … `docs/06_Business.md` — maintenance docs; `docs/07_Plan.*` — delivery plan
- `docs/ARCHITECTURE.md` — deck navigation, full-page animation, layout system, platform connections
- `docs/COMPONENTS.md` — component/page reference
- `docs/GUIDES.md` — how-tos + gotchas (read this before changing animation or tables)
- `docs/CHANGELOG.md` — frontend build history
- `docs/prompts/` — all `.prompt.md` sources (git-ignored, kept locally)
- `app/README.md` — frontend: design system, structure, run instructions
- `server/README.md` — API reference, data model, validation, production path
- `app/CODEMAP.md` — suggested code-reading order

Research heavies (`.pdf` / `.typ` / `.prompt.md`) are git-ignored build/sources —
kept in your working copy, regenerated from the `.md` files (e.g.
`typst compile --root . docs/07_Plan.typ docs/07_Plan.pdf`).
