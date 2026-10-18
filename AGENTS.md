# AGENTS.md — Huuk (Content Planner)

Vue 3 frontend + Rust (Axum) backend + PostgreSQL (SQLx) + Caddy + ts-rs + Zod (Master Engineering SOP compliant).
`app/` canonical frontend, `server/` API, `deploy/` & `ops/` deploy configs/scripts, `docs/` maintenance docs.

## Run / verify

```sh
just ci              # Full SOP gate: cargo fmt, clippy -D warnings, ts-rs contract check, cargo test, bun typecheck/test/build
just dev-db          # Start local PostgreSQL 16 container (deploy/docker-compose.dev.yml)
just gen-types       # Export TypeScript bindings from Rust structs via ts-rs
just check-types     # Verify app/src/api/types/ matches Rust structs
bun run dev          # backend :8787 (DEMO_MODE=1) + Vite, wired via VITE_API_URL; Ctrl+C stops both
bun run dev:mock     # frontend only, in-memory mock (no Rust needed)
bun run dev-server   # backend only
bun run test         # cargo test (server) + vitest run (app)
bun run build        # cargo build --release + vue-tsc -b && vite build
```

- `dev.sh` **requires bun** (`command -v bun` or it exits) — README's "npm otherwise"
  does not apply to `dev`/`dev-mock`; plain `npm run` only works for single-side commands.
  First frontend install: `cd app && bun install` (single lockfile: `app/bun.lock`).
- Backend port: `PORT=…` (default `8787`, bind `BIND_ADDR`, default `127.0.0.1`).
  `dev.sh` reuses an already-running backend on that port instead of rebuilding.
- Focused tests: `cargo test --manifest-path server/Cargo.toml <filter>` (substring match;
  suite is `server/tests/api.rs`, boots the real router via `tower::oneshot`, no network);
  `cd app && bun run test -- <file-or-substring>` (vitest filter). Typecheck only:
  `cd app && bun run typecheck`.
- Before calling a change done: run `just ci` (`cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `just check-types`, `cargo test`, `bun run typecheck`, `bun run test`, `bun run build`).

## Data-source swap & Contract Sync (read before touching data flow)

`app/src/api/index.ts` picks the source on `VITE_API_URL`: unset → `src/mock/api.ts`
(in-memory, resets on reload); set → `src/api/http.ts` (using `src/api/client.ts` + `src/api/schemas/index.ts`) → Rust API. Both implement the same functions. Rules:

- Pages/components **never import `src/mock/` directly**; only via `src/api/index.ts`.
- DTOs in `server/src/model.rs` and `ApiErrorResponse` in `server/src/error.rs` derive `ts_rs::TS` (`#[ts(export, export_to = "../../app/src/api/types/")]`) and are paired with Zod schemas and compile-time `AssertEqual<z.infer<typeof Schema>, TsRsType>` assertions in `app/src/api/schemas/index.ts`.
- Never use `as unknown as` in `app/src/` or `.unwrap()` / `.expect()` in production `server/src/`.
- New/changed endpoint = `server/src/handlers.rs` → `app/src/api/http.ts` + `app/src/api/schemas/index.ts` **and** `app/src/mock/api.ts` → `server/openapi.yaml`
  (source of truth, served at `/api/openapi.yaml` + Swagger UI at `/api/docs`;
  `tests/api.rs` asserts both are served) → tests on both sides.
- JSON is camelCase; errors follow `ApiErrorResponse` (`{ "error_code": "…", "message": "…", "details": …, "error": "…" }`). Restart Vite after
  adding/changing `.env.local` (`VITE_API_URL=http://localhost:8787`).
- Stale UI on `:8080` is expected: that's the **built** SPA from the deploy stack, not
  the dev server. After frontend changes: `cd app && bun run build`, then redeploy/restart.

## Backend notes (`server/src/`)

- `main.rs` (`mimalloc` global allocator, config/env, PostgreSQL pool + SQLx migrations + snapshot persistence loop, graceful shutdown); `lib.rs` router + `AppState`
  (`Arc<RwLock<Store>>`) + `/healthz`; `db.rs` PostgreSQL pool (`sqlx::PgPool`, `statement_timeout = 5000ms`, `lock_timeout = 3000ms`), migrations (`server/migrations/`), and DB snapshot persistence; `store.rs` state/seed/snapshots; `model.rs` types (`ts-rs`); `handlers.rs`
  one handler per endpoint (never hold `RwLock` guards across `.await` points); `error.rs` `ApiErrorResponse` envelope; `oauth.rs`, `live.rs`, `ads.rs`, `secrets.rs`.
- Local dev runs `DEMO_MODE=1` (seeded accounts, auth optional, open registration).
  Without it: production mode — auth required, demo accounts gone, registration closed,
  first Owner from `ADMIN_EMAIL`/`ADMIN_PASSWORD`.
- Auth: Argon2id (`tokio::task::spawn_blocking`), 64-hex-char session tokens, 7-day TTL.
  The browser session is an **HttpOnly `cp_session` cookie** (`axum-extra` `Cookie` builder with `Secure` + `SameSite::Lax`;
  `COOKIE_SECURE=0` opts out for plain-HTTP LAN testing) — the SPA must never store
  tokens (no localStorage). API clients may use the `token` from login/register as
  `Authorization: Bearer`; `POST /api/auth/refresh` rotates the session (cookie re-issued;
  Bearer callers get the new token in the body). The frontend calls it single-flight on
  401 in `api/client.ts` (`isRefreshing` + `failedQueue`).
  With `authRequired` on, post locks derive the holder from the **session** — client-sent
  `user`/`lockedBy` are ignored; creates never accept `lockedBy`.
- State persists to PostgreSQL (`DATABASE_URL` via `db.rs`) and/or `DATA_FILE` atomically every 5s when dirty + on SIGTERM/SIGINT.
  `data/`, `backups/`, `ops/deploy/.env.production`, `*.prompt.md`, `*.pdf`/`*.typ`
  are git-ignored runtime/secrets — never commit them.
- Demo logins: `owner@studio.local` / `editor@studio.local` / `client@studio.local`,
  password `demo1234`. The session lives in the HttpOnly cookie — never in web storage.

## Frontend rules (`app/src/`)

Non-negotiables (from `app/README.md`): screens register **only** in `core/screens.ts`
(order = deck order); only `PlannerPage` writes posts; computed-field edits go through
`ProtectedModal`; navigation is a slide deck (URL follows the slide); import via `@`
alias (`@/*` → `src/*`), never long `../../` chains.

- **Components are vendored**: `app/src/components/` (barrel at `components/ui/index.ts`)
  is this repo's source of truth. Never import from the sibling Pugu kit
  (`../../components/huuk`) — port the file over instead.
- **Styling is Tailwind v4** (`@tailwindcss/vite` in `app/vite.config.ts`, tokens in
  `@theme` at the top of `src/app/style.css`). Colors are `var(--*)` aliases so
  runtime brand theming (`core/theme.ts` → `:root`) keeps working — never hardcode
  values Tailwind already maps. Type scale: `text-2xs(11)–text-5xl(26)` all with
  `line-height: 1.6`; sizes outside it → exact `text-[..]`. Radii: only
  `rounded-none`/`rounded-full`, never bare `rounded`. Borders always carry a color
  (`border border-line` — preflight defaults to `currentColor`). Every `<button>`
  gets explicit `w-full` or `w-auto`; plain `input/select/textarea` need no classes
  (base element skin). Never put `text-*` on `h1–h4` without a paired `leading-*`.
  The UNLAYERED shell in `style.css` (deck full-mode, `*-scroll` wide panes,
  container queries, grid primitives) beats utilities by design — keep its hook
  classes in markup (`deck/slide/spacer/slide-card/card-scroll/grid*/tbl/row/
  *-page/*-scroll/p-drawer*`) and never add conflicting layout utilities on them.
  No `<style>` blocks in components except `@keyframes`/`@container` (Drawer).
  tests query by role/text/tag — never by styling classes. Provider login pages
  (`PlatformLogin`) keep their own chrome, never themed.
- **TanStack Table is v9, not v8**: `useTable` + `tableFeatures` + `create*RowModel`;
  `useVueTable`/`getCoreRowModel` do not exist. Column type is
  `ColumnDef<typeof features, Row, unknown>[]` (features first); row shapes are `type`
  aliases, not `interface`s; pass the `computed` ref itself (not `.value`) to `useTable`;
  numeric columns sort desc-first unless `sortFn: 'basic'`.
- Animation: animate real `width`/`left` on a pinned (`position:fixed`) card, never
  `scaleX`/`clip-path`; guard re-entry (`fullBusy` / `fullAnim?.stop()`). Deck drags:
  capture pointer only after >8px movement, disable snap/smooth mid-drag.
- New query/mutation in `core/queries.ts` must `invalidateQueries` every affected key on
  success (symptom of a miss: write succeeds, UI doesn't refresh).

## Docs that matter

`docs/GUIDES.md` (recipes + Gotchas table — check before changing animation/tables/deck),
`docs/ARCHITECTURE.md` (deck, layout, animation), `server/README.md` (full API +
production env reference), `docs/DEPLOYMENT.md` + `ops/README.md` (deploy/backup/smoke;
entry points are `ops/scripts/{dev,deploy,install,backup,restore,smoke}.sh`, `Makefile`
is a thin front door). The binary serves the built SPA itself via `STATIC_DIR=app/dist`.
